#[macro_use]
extern crate log;
extern crate env_logger;

use std::convert::TryFrom;

fn mkpattern(pos: usize, len: usize) -> Vec<i8> {
  debug_assert_eq!(pos > 0, true);
  let base = [0, 1, 0, -1];
  let mut result = Vec::with_capacity(len);
  let mut is_first = true;
  loop {
    if is_first {
      is_first = false;
      // whole list is shifted by one to the left
      for _ in 1..pos {
        result.push(base[0]);
        if result.len() == len {
          return result;
        }
      }
      // other elemnets are just like in the regular case
      for i in 1..base.len() {
        for _ in 0..pos {
          result.push(base[i]);
          if result.len() == len {
            return result;
          }
        }
      }
      continue;
    }

    for i in 0..base.len() {
      for _ in 0..pos {
        result.push(base[i]);
        if result.len() == len {
          return result;
        }
      }
    }
  }
}

fn split_input(s: &str) -> Vec<i32> {
  s.chars()
    .map(|c| i32::try_from(c.to_digit(10).expect("Invalid digit")).unwrap())
    .collect()
}

fn apply_pattern(src: &Vec<i32>, count: usize) -> Vec<i32> {
  let mut input = src.clone();
  let n = input.len();
  debug!("Input size: {:?}", input.len());
  for phase in 1..=count {
    debug!("Phase {} of {}", phase, count);
    debug!("Input: {:?}", input);
    /*
     Each element in the new list is built by multiplying every value in the
     input list by a value in a repeating pattern and then adding up the results.
     So, if the input list were 9, 8, 7, 6, 5 and the pattern for a given element
     were 1, 2, 3, the result would be 9*1 + 8*2 + 7*3 + 6*1 + 5*2 (with each
     input element on the left and each value in the repeating pattern on the
     right of each multiplication). Then, only the ones digit is kept: 38 becomes
     8, -17 becomes 7, and so on.
    */
    let mut new_input = vec![0; n];
    for j in 0..n {
      let pattern = mkpattern(j + 1, n);
      let mut val = 0;
      for l in 0..n {
        let from_pattern = pattern[l] as i32;
        let from_input = input[l];
        val += from_pattern * from_input;
      }
      new_input[j] = val.abs() % 10;
    }
    input = new_input;
  }
  return input;
}

fn fast_fft(input: &mut [i32], iterations: usize) {
  let n = input.len();
  debug!("iterations: {}, n={}", iterations, n);
  let last = n - 1;
  for _ in 1..=iterations {
    for i in 1..=last {
      let pos = last - i;
      input[pos] = (input[pos + 1] + input[pos]).abs() % 10;
    }
  }
  /* For i >= floor(n/2):
   * output[end] = input[end]
   * output[end - 1] = input[end] + input[end - 1]
   * output[end - 2] = input[end] + input[end - 1] + input[end - 2]
   */
}

fn part_two(s: &str) -> i32 {
  let prefix: Vec<i32> = s
    .chars()
    .map(|c| i32::try_from(c.to_digit(10).expect("Invalid digit")).unwrap())
    .collect();

  let mut offset = prefix[0];
  for i in 1..=6 {
    offset = offset * 10 + prefix[i];
  }
  let offset = offset as usize;

  let factor = 10000;
  let mut huge_s = String::with_capacity(s.len() * factor);
  for _ in 0..factor {
    huge_s.push_str(s);
  }

  let mut real_input: Vec<i32> = huge_s
    .chars()
    .skip(offset)
    .map(|c| i32::try_from(c.to_digit(10).expect("Invalid digit")).unwrap())
    .collect();
  fast_fft(real_input.as_mut(), 100);

  let mut result = real_input[0];
  for i in 1..=7 {
    result = result * 10 + real_input[i];
  }
  return result;
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn mkpattern_test() {
    init();
    assert_eq!(mkpattern(3, 12), vec![0, 0, 1, 1, 1, 0, 0, 0, -1, -1, -1, 0]);
    assert_eq!(mkpattern(2, 15), vec![0, 1, 1, 0, 0, -1, -1, 0, 0, 1, 1, 0, 0, -1, -1]);
  }

  #[test]
  fn apply_pattern_test() {
    init();
    let expected = vec![
      vec![4, 8, 2, 2, 6, 1, 5, 8],
      vec![3, 4, 0, 4, 0, 4, 3, 8],
      vec![0, 3, 4, 1, 5, 5, 1, 8],
      vec![0, 1, 0, 2, 9, 4, 9, 8],
    ];
    let src = &"12345678"[..];
    let input = split_input(src);
    for i in 0..expected.len() {
      let result = apply_pattern(&input, i + 1);
      assert_eq!(result, expected[i]);
    }
  }

  fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
  }

  #[test]
  fn fast_fft_test() {
    init();
    let expected = vec![[6, 1, 5, 8], [0, 4, 3, 8], [5, 5, 1, 8], [9, 4, 9, 8]];
    let src = &"12345678"[..];
    let mut input = split_input(src);
    let input = &mut input[4..];
    debug!("Input: {:?}", input);
    for i in 0..expected.len() {
      fast_fft(input, 1);
      assert_eq!(input, expected[i]);
    }

    let mut input = split_input(src);
    let input = &mut input[4..];
    fast_fft(input, 4);
    assert_eq!(input, expected[3]);
  }

  #[test]
  fn part_two_test() {
    init();
    assert_eq!(part_two(&"03036732577212944063491565474664"[..]), 84462026);
    assert_eq!(part_two(&"02935109699940807407585447034323"[..]), 78725270);
    assert_eq!(part_two(&"03081770884921959731165446850517"[..]), 53553731);
  }
}

fn main() {
  env_logger::init();

  let challenge_input = &"59718730609456731351293131043954182702121108074562978243742884161871544398977055503320958653307507508966449714414337735187580549358362555889812919496045724040642138706110661041990885362374435198119936583163910712480088609327792784217885605021161016819501165393890652993818130542242768441596060007838133531024988331598293657823801146846652173678159937295632636340994166521987674402071483406418370292035144241585262551324299766286455164775266890428904814988362921594953203336562273760946178800473700853809323954113201123479775212494228741821718730597221148998454224256326346654873824296052279974200167736410629219931381311353792034748731880630444730593"[..];

  let input = split_input(challenge_input);
  let result = apply_pattern(&input, 100);
  let prefix: Vec<i32> = result.iter().take(8).map(|x| *x).collect();
  let mut prefix_str = String::with_capacity(prefix.len());
  for d in prefix.iter() {
    prefix_str.push_str(&d.to_string());
  }
  println!("Part One: {:}", prefix_str);

  let result = part_two(challenge_input);
  println!("Part Two: {:}", result);
}
