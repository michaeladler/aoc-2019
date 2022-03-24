use log::debug;
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct CompressionResult {
    pub main: Vec<String>,
    pub prog_a: Vec<String>,
    pub prog_b: Vec<String>,
    pub prog_c: Vec<String>,
}

fn find_part<'a>(
    main: &'a [String],
    blacklist: &HashSet<&str>,
    letter: &str,
    subsequences: &mut HashMap<&[String], usize>,
) -> (Vec<String>, &'a [String]) {
    let mut start = 0;
    if !blacklist.is_empty() {
        for c in main {
            if !blacklist.contains(&c[..]) {
                break;
            }
            start += 1;
        }
    }
    debug_assert!((!blacklist.is_empty() && start > 0) || (blacklist.is_empty() && start == 0));
    debug!("start_{}: {}", letter, start);

    let mut part = None;
    let n = main.len();
    for i in start + 1..=std::cmp::min(start + 10, n) {
        let cand = &main[start..i];
        if subsequences.contains_key(cand) {
            part = Some(cand);
        }
    }
    let part = part.unwrap();
    debug_assert!(part.len() <= 10 && part.len() >= 2 && part.len() % 2 == 0);
    debug!("part_{}: {}", letter, part.join(","));
    subsequences.remove(part);

    let main = main.join(",").replace(&part.join(","), letter);
    debug!("main: {}", main);
    let main: Vec<String> = main.split(",").map(|x| x.to_string()).collect();

    return (main, part);
}

fn longest_even_subsequences(
    s: &[String],
    min_len: usize,
    max_len: usize,
) -> HashMap<&[String], usize> {
    let mut result = HashMap::new();
    let n = s.len();
    for small_i in 0..=n / 2 {
        let i = 2 * small_i;
        for j in i + 1..=std::cmp::min(n, i + max_len) {
            if (j - i) % 2 != 0 || j - i < min_len {
                continue;
            }
            let substr = &s[i..j];
            let mut tmp = s[j..n].join(",");
            let mut count = 1;
            let mut old_n = tmp.len();
            loop {
                tmp = tmp.replacen(&substr.join(","), &"", 1);
                let new_n = tmp.len();
                if new_n == old_n {
                    break;
                }
                old_n = new_n;
                count += 1;
            }
            if count > 1 {
                if let Some(existing) = result.get_mut(substr) {
                    if count > *existing {
                        *existing = count;
                    }
                } else {
                    result.insert(substr, count);
                }
            }
        }
    }
    return result;
}

pub fn compress(path: &[String]) -> Option<CompressionResult> {
    debug!("Compressing path: {}", path.join(","));

    let mut subsequences = longest_even_subsequences(&path, 4, 10);
    let mut blacklist = HashSet::new();

    let (main, part_a) = find_part(&path, &blacklist, &"A"[..], &mut subsequences);
    blacklist.insert("A");
    let (main, part_b) = find_part(&main, &blacklist, &"B"[..], &mut subsequences);
    blacklist.insert("B");
    let (main, part_c) = find_part(&main, &blacklist, &"C"[..], &mut subsequences);

    return Some(CompressionResult {
        main,
        prog_a: part_a.iter().map(|x| x.to_string()).collect(),
        prog_b: part_b.iter().map(|x| x.to_string()).collect(),
        prog_c: part_c.iter().map(|x| x.to_string()).collect(),
    });
}

#[test]
fn test_longest_subsequences() {
    let s: Vec<&str> = "A,X,B,B,B,C,B,B".split(",").collect();
    let s2: Vec<String> = s.iter().map(|x| x.to_string()).collect();
    let result = longest_even_subsequences(&s2, 0, 2);

    println!("{:?}", result);
    assert_eq!(2, *result.get(&s2[2..=3]).unwrap());
}

#[test]
fn compress_test_small() {
    let s: Vec<String> = "R,8,R,8,R,4,R,4,R,8,L,6,L,2,R,4,R,4,R,8,R,8,R,8,L,6,L,2"
        .split(",")
        .map(|x| x.to_string())
        .collect();
    assert!(compress(&s).is_some());
}

#[test]
fn compress_test_large() {
    let s: Vec<String> = "R,8,L,12,R,8,R,12,L,8,R,10,R,12,L,8,R,10,R,8,L,12,R,8,R,8,L,8,L,8,R,8,R,10,R,8,L,12,R,8,R,8,L,12,R,8,R,8,L,8,L,8,R,8,R,10,R,12,L,8,R,10,R,8,L,8,L,8,R,8,R,10"
        .split(",")
        .map(|x| x.to_string()).collect();
    let result = compress(&s).unwrap();
    let expected: Vec<String> = "A,B,B,A,C,A,A,C,B,C"
        .split(",")
        .map(|x| x.to_string())
        .collect();
    assert_eq!(expected, result.main);
}
