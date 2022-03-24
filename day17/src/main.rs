mod compress;
mod grid;

use std::collections::HashSet;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::time::Instant;

use log::{debug, trace};
use pretty_env_logger;

use crate::compress::compress;
use crate::grid::{Direction, Field, Grid};

use aoc2019::intcode::{IntcodeProgram, IntcodeResult};

#[derive(Debug, PartialEq)]
pub struct Robot {
    pub x: i64,
    pub y: i64,
    pub direction: Direction,
}

impl Robot {
    fn rotate(&mut self, direction: Direction) -> Option<&str> {
        use Direction::*;
        let result = match (self.direction, direction) {
            (North, East) | (East, South) | (South, West) | (West, North) => Some("R"),
            (North, West) | (West, South) | (South, East) | (East, North) => Some("L"),
            _ => None,
        };
        if result.is_some() {
            self.direction = direction;
        }
        return result;
    }
}

/// `greedy_path` attempts to visit every tile by following a very simple strategy:
/// While there are unvisited tiles:
///   walk in a straight line to the furthest node and continue from there
fn greedy_path(grid: &Grid, robot: &mut Robot) -> Vec<String> {
    let mut visited: HashSet<(i64, i64)> = HashSet::new();

    debug!("start: {:?}", robot);
    let mut path = Vec::new();

    let mut go = true;
    while go {
        use Direction::*;

        // choose the furthest point
        let mut next = None;
        let mut direction = None;
        let mut max_dist = 0;

        for (dir, p) in &[
            (East, grid.walk_east(robot.x, robot.y)),
            (West, grid.walk_west(robot.x, robot.y)),
        ] {
            if !visited.contains(p) {
                let d = (robot.x - p.0).abs();
                if d > max_dist {
                    max_dist = d;
                    next = Some(*p);
                    direction.replace(dir.clone());
                }
            }
        }
        for (dir, p) in &[
            (North, grid.walk_north(robot.x, robot.y)),
            (South, grid.walk_south(robot.x, robot.y)),
        ] {
            if !visited.contains(p) {
                let d = (robot.y - p.1).abs();
                if d > max_dist {
                    max_dist = d;
                    next = Some(*p);
                    direction.replace(dir.clone());
                }
            }
        }

        match next {
            Some(n) => {
                trace!(
                    "next: {:?}, direction: {:?}, dist: {}",
                    n,
                    direction,
                    max_dist
                );
                visited.insert(n);
                let orient = robot.rotate(direction.unwrap()).unwrap();
                path.push(String::from(orient));
                path.push(max_dist.to_string());
                robot.x = n.0;
                robot.y = n.1;
            }
            None => {
                go = false;
            }
        }
    }

    return path;
}

fn read_input(fname: &str) -> Vec<i64> {
    let file = File::open(fname).unwrap();
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents).unwrap();
    let code: Vec<i64> = contents
        .split(",")
        .map(|x| x.trim_end().parse::<i64>().unwrap())
        .collect();
    return code;
}

fn parse_ascii(src: Vec<i64>) -> (Robot, Grid) {
    let mut grid: Vec<Vec<Field>> = Vec::new();
    let mut row: Vec<Field> = Vec::new();

    let mut robot = None;
    let mut x = 0;
    let mut y = 0;
    for d in src.iter() {
        let ch = char::from(*d as u8);
        match ch {
            '\n' => {
                grid.push(row);
                row = Vec::new();
                x = 0;
                y += 1;
                continue;
            }
            '#' => {
                row.push(Field::Tile);
            }
            '.' => {
                row.push(Field::Empty);
            }
            c => {
                row.push(Field::Tile);
                robot = Some(Robot {
                    x,
                    y,
                    direction: match c {
                        '^' => Direction::North,
                        'v' => Direction::South,
                        '>' => Direction::East,
                        '<' => Direction::West,
                        _ => panic!("Unexpected ASCII char"),
                    },
                });
            }
        }
        x += 1;
    }

    let height = grid.len();
    let width = grid[0].len();
    return (
        robot.unwrap(),
        Grid {
            fields: grid,
            width,
            height,
        },
    );
}

fn build(code: Vec<i64>) -> (Robot, Grid) {
    let mut program = IntcodeProgram::new(code);
    let mut output = Vec::new();
    program.run(&mut Vec::new(), &mut output);
    return parse_ascii(output);
}

fn part1() -> usize {
    let code = read_input(&"input.txt"[..]);
    let (robot, grid) = build(code);
    debug!("Grid:\n{:}", grid);
    debug!("Robot: {:?}", robot);
    return grid.intersections().iter().map(|x| x.0 * x.1).sum();
}

fn to_opcode_input(prog: &Vec<String>) -> Vec<i64> {
    let mut result: Vec<i64> = Vec::new();
    let comma = 44;
    let newline = 10;
    for s in prog {
        for c in s.chars() {
            result.push((c as u8) as i64);
        }
        result.push(comma);
    }
    result.pop(); // pop last comma
    result.push(newline);
    return result;
}

fn part2() -> i64 {
    let mut code = read_input(&"input.txt"[..]);
    let (mut robot, grid) = build(code.clone());
    let path = greedy_path(&grid, &mut robot);

    let compressed = compress(path.as_slice()).expect("Compression failed");
    debug!("Compressed: {:?}", compressed);

    let mut input_src = to_opcode_input(&compressed.main);
    let mut prog_a = to_opcode_input(&compressed.prog_a);
    let mut prog_b = to_opcode_input(&compressed.prog_b);
    let mut prog_c = to_opcode_input(&compressed.prog_c);
    let mut visual_feed = vec![('n' as u8) as i64, 10];

    input_src.append(&mut prog_a);
    input_src.append(&mut prog_b);
    input_src.append(&mut prog_c);
    input_src.append(&mut visual_feed);
    let mut output = Vec::new();

    // Force the vacuum robot to wake up by changing the value in your ASCII program at address 0 from 1 to 2
    debug_assert_eq!(1, code[0]);
    code[0] = 2;

    let mut program = IntcodeProgram::new(code);
    debug!(
        "Opcode input: {}",
        input_src
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(",")
    );
    let status = program.run(&input_src, &mut output);
    debug_assert_eq!(IntcodeResult::TERMINATED, status);

    debug!(
        "Program finished with output: {}",
        output
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(",")
    );
    // report the amount of space dust it collected as a large, non-ASCII value in a single output instruction
    let result: Vec<i64> = output
        .iter()
        .map(|x| *x)
        .filter(|x| *x >= 128)
        .take(1)
        .collect();
    return result[0];
}

fn main() {
    pretty_env_logger::init();
    let start = Instant::now();
    let p1 = part1();
    let p2 = part2();
    let elapsed = start.elapsed();
    println!(
        "Part One: {}\n\
         Part Two: {}\n\
         Total duration: {}ms",
        p1,
        p2,
        elapsed.as_millis()
    );
}

#[test]
fn part1_test() {
    assert_eq!(4408, part1());
}

#[test]
fn part2_test() {
    assert_eq!(862452, part2());
}

#[test]
fn test_to_opcode_input_main() {
    let input = "A,B,C,B,A,C".split(",").map(|x| x.to_string()).collect();
    let output = to_opcode_input(&input);
    assert_eq!(vec![65, 44, 66, 44, 67, 44, 66, 44, 65, 44, 67, 10], output)
}

#[test]
fn test_to_opcode_input_sub() {
    let input = "R,4,R,4,R,8".split(",").map(|x| x.to_string()).collect();
    let output = to_opcode_input(&input);
    assert_eq!(vec![82, 44, 52, 44, 82, 44, 52, 44, 82, 44, 56, 10], output)
}

#[test]
fn test_to_opcode_input_large_num() {
    let input = "R,123".split(",").map(|x| x.to_string()).collect();
    let output = to_opcode_input(&input);
    assert_eq!(vec![82, 44, 49, 50, 51, 10], output)
}
