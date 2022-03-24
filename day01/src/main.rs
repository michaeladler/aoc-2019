mod solution;

use std::env;
use std::time::Instant;

use solution::{part1, part2};

fn main() {
    let mut solve_one = true;
    let mut solve_two = true;

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
        let start = Instant::now();
        let answer = part1();
        let elapsed = start.elapsed();
        println!("Part 1 (solved in {}ms): {}", elapsed.as_millis(), answer,);
    };
    if solve_two {
        let start = Instant::now();
        let answer = part2();
        let elapsed = start.elapsed();
        println!("Part 2 (solved in {}ms): {}", elapsed.as_millis(), answer,);
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn part1_test() {
        assert_eq!(3455717, part1());
    }

    #[test]
    fn part2_test() {
        assert_eq!(5180690, part2());
    }
}
