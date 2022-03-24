mod maze;

use std::env;
use std::time::Instant;

use env_logger;
use log::{debug, info};

use crate::maze::{read_input, Point, Portal};

fn part1(fname: &str) -> usize {
    let grid = read_input(fname);
    let start = grid.find_portals(&Portal::new('A', 'A'))[0];
    debug!("start at: {:?}", start);

    let parents = grid.bfs(start);

    let finish = grid.find_portals(&Portal::new('Z', 'Z'))[0];
    debug!("finish at: {:?}", start);

    // build path
    let mut path: Vec<Point> = Vec::new();
    let mut current = finish;
    path.push(current);
    while let Some(parent) = parents.get(&current) {
        path.push(*parent);
        current = *parent;
    }

    let answer = path.len() - 1;
    return answer;
}

fn part2(fname: &str) -> usize {
    let grid = read_input(fname);
    let start = grid.find_portals(&Portal::new('A', 'A'))[0];
    return grid.recursive_walk(start);
}

fn main() {
    env_logger::init();

    let mut solve_one = true;
    let mut solve_two = true;

    let fname = if let Some(arg) = env::args().nth(1) {
        arg
    } else {
        String::from(&"input.txt"[..])
    };
    info!("Using file: {}", fname);

    if let Some(part) = env::args().nth(2) {
        if part == "1" {
            solve_two = false;
        } else if part == "2" {
            solve_one = false;
        } else {
            panic!("invalid choice");
        }
    }

    if solve_one {
        info!("Solving part 1");
        let start = Instant::now();
        let answer = part1(&fname);
        let elapsed = start.elapsed();
        println!("Part 1 (solved in {}ms): {}", elapsed.as_millis(), answer,);
    };
    if solve_two {
        info!("Solving part 2");
        let start = Instant::now();
        let answer = part2(&fname);
        let elapsed = start.elapsed();
        println!("Part 2 (solved in {}ms): {}", elapsed.as_millis(), answer,);
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn part1_small_test() {
        assert_eq!(23, part1(&"small.txt"[..]));
    }

    #[test]
    fn part1_small2_test() {
        assert_eq!(58, part1(&"small2.txt"[..]));
    }

    #[test]
    fn part1_test() {
        assert_eq!(674, part1(&"input.txt"[..]));
    }

    #[test]
    fn part2_small_test() {
        assert_eq!(26, part2(&"small.txt"[..]));
    }

    #[test]
    fn part2_small3_test() {
        assert_eq!(396, part2(&"small3.txt"[..]));
    }

    #[test]
    fn part2_test() {
        assert_eq!(7636, part2(&"input.txt"[..]));
    }
}
