mod bitset;
mod graph;
mod grid;
mod part1;
mod part2;

use env_logger;
use log::info;
use std::env;
use std::time::Instant;

use part1::part1;
use part2::part2;

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
