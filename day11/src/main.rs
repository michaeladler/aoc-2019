#[macro_use]
extern crate log;
extern crate aoc2019;
extern crate env_logger;

use std::collections::HashMap;

use aoc2019::intcode::{IntcodeProgram, IntcodeResult};

#[derive(Debug, PartialEq, Copy, Clone)]
enum Color {
    Black = 0,
    White = 1,
}

#[derive(Debug, PartialEq)]
enum Turn {
    Left,
    Right,
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum Direction {
    North,
    East,
    South,
    West,
}

fn update_direction(d: Direction, inst: Turn) -> Direction {
    match (d, inst) {
        (Direction::North, Turn::Left) => Direction::West,
        (Direction::East, Turn::Left) => Direction::North,
        (Direction::South, Turn::Left) => Direction::East,
        (Direction::West, Turn::Left) => Direction::South,
        (Direction::North, Turn::Right) => Direction::East,
        (Direction::East, Turn::Right) => Direction::South,
        (Direction::South, Turn::Right) => Direction::West,
        (Direction::West, Turn::Right) => Direction::North,
    }
}

#[derive(Debug, PartialEq)]
struct Panel {
    color: Color,
    visited: usize,
}

impl Panel {
    pub fn new() -> Self {
        Self {
            color: Color::Black,
            visited: 0,
        }
    }
}

fn paint(starting_color: Color) -> HashMap<(i32, i32), Panel> {
    let code = vec![
        3,
        8,
        1005,
        8,
        311,
        1106,
        0,
        11,
        0,
        0,
        0,
        104,
        1,
        104,
        0,
        3,
        8,
        102,
        -1,
        8,
        10,
        1001,
        10,
        1,
        10,
        4,
        10,
        1008,
        8,
        0,
        10,
        4,
        10,
        1002,
        8,
        1,
        29,
        3,
        8,
        102,
        -1,
        8,
        10,
        1001,
        10,
        1,
        10,
        4,
        10,
        108,
        0,
        8,
        10,
        4,
        10,
        101,
        0,
        8,
        50,
        1,
        2,
        19,
        10,
        1006,
        0,
        23,
        1,
        103,
        14,
        10,
        1,
        1106,
        15,
        10,
        3,
        8,
        1002,
        8,
        -1,
        10,
        1001,
        10,
        1,
        10,
        4,
        10,
        1008,
        8,
        1,
        10,
        4,
        10,
        102,
        1,
        8,
        88,
        1006,
        0,
        59,
        3,
        8,
        1002,
        8,
        -1,
        10,
        101,
        1,
        10,
        10,
        4,
        10,
        1008,
        8,
        1,
        10,
        4,
        10,
        1002,
        8,
        1,
        113,
        2,
        101,
        12,
        10,
        2,
        1001,
        0,
        10,
        2,
        1006,
        14,
        10,
        3,
        8,
        1002,
        8,
        -1,
        10,
        101,
        1,
        10,
        10,
        4,
        10,
        108,
        0,
        8,
        10,
        4,
        10,
        102,
        1,
        8,
        146,
        1,
        1106,
        11,
        10,
        1006,
        0,
        2,
        1,
        9,
        8,
        10,
        3,
        8,
        1002,
        8,
        -1,
        10,
        1001,
        10,
        1,
        10,
        4,
        10,
        1008,
        8,
        1,
        10,
        4,
        10,
        101,
        0,
        8,
        180,
        1,
        6,
        13,
        10,
        1,
        1102,
        15,
        10,
        2,
        7,
        1,
        10,
        3,
        8,
        1002,
        8,
        -1,
        10,
        1001,
        10,
        1,
        10,
        4,
        10,
        108,
        0,
        8,
        10,
        4,
        10,
        1002,
        8,
        1,
        213,
        1006,
        0,
        74,
        2,
        1005,
        9,
        10,
        3,
        8,
        1002,
        8,
        -1,
        10,
        101,
        1,
        10,
        10,
        4,
        10,
        1008,
        8,
        0,
        10,
        4,
        10,
        1002,
        8,
        1,
        243,
        3,
        8,
        1002,
        8,
        -1,
        10,
        101,
        1,
        10,
        10,
        4,
        10,
        108,
        1,
        8,
        10,
        4,
        10,
        101,
        0,
        8,
        264,
        2,
        104,
        8,
        10,
        3,
        8,
        1002,
        8,
        -1,
        10,
        1001,
        10,
        1,
        10,
        4,
        10,
        108,
        1,
        8,
        10,
        4,
        10,
        1001,
        8,
        0,
        290,
        101,
        1,
        9,
        9,
        1007,
        9,
        952,
        10,
        1005,
        10,
        15,
        99,
        109,
        633,
        104,
        0,
        104,
        1,
        21101,
        387512640296,
        0,
        1,
        21101,
        0,
        328,
        0,
        1106,
        0,
        432,
        21102,
        1,
        665749660564,
        1,
        21101,
        339,
        0,
        0,
        1106,
        0,
        432,
        3,
        10,
        104,
        0,
        104,
        1,
        3,
        10,
        104,
        0,
        104,
        0,
        3,
        10,
        104,
        0,
        104,
        1,
        3,
        10,
        104,
        0,
        104,
        1,
        3,
        10,
        104,
        0,
        104,
        0,
        3,
        10,
        104,
        0,
        104,
        1,
        21102,
        179318226984,
        1,
        1,
        21101,
        386,
        0,
        0,
        1105,
        1,
        432,
        21101,
        46266346499,
        0,
        1,
        21101,
        0,
        397,
        0,
        1105,
        1,
        432,
        3,
        10,
        104,
        0,
        104,
        0,
        3,
        10,
        104,
        0,
        104,
        0,
        21102,
        709580555028,
        1,
        1,
        21102,
        420,
        1,
        0,
        1106,
        0,
        432,
        21102,
        1,
        988220642068,
        1,
        21101,
        0,
        431,
        0,
        1106,
        0,
        432,
        99,
        109,
        2,
        21202,
        -1,
        1,
        1,
        21101,
        40,
        0,
        2,
        21102,
        1,
        463,
        3,
        21102,
        1,
        453,
        0,
        1106,
        0,
        496,
        109,
        -2,
        2106,
        0,
        0,
        0,
        1,
        0,
        0,
        1,
        109,
        2,
        3,
        10,
        204,
        -1,
        1001,
        458,
        459,
        474,
        4,
        0,
        1001,
        458,
        1,
        458,
        108,
        4,
        458,
        10,
        1006,
        10,
        490,
        1102,
        0,
        1,
        458,
        109,
        -2,
        2105,
        1,
        0,
        0,
        109,
        4,
        2102,
        1,
        -1,
        495,
        1207,
        -3,
        0,
        10,
        1006,
        10,
        513,
        21101,
        0,
        0,
        -3,
        21201,
        -3,
        0,
        1,
        22101,
        0,
        -2,
        2,
        21102,
        1,
        1,
        3,
        21101,
        532,
        0,
        0,
        1106,
        0,
        537,
        109,
        -4,
        2106,
        0,
        0,
        109,
        5,
        1207,
        -3,
        1,
        10,
        1006,
        10,
        560,
        2207,
        -4,
        -2,
        10,
        1006,
        10,
        560,
        22102,
        1,
        -4,
        -4,
        1105,
        1,
        628,
        21201,
        -4,
        0,
        1,
        21201,
        -3,
        -1,
        2,
        21202,
        -2,
        2,
        3,
        21102,
        1,
        579,
        0,
        1105,
        1,
        537,
        22101,
        0,
        1,
        -4,
        21101,
        1,
        0,
        -1,
        2207,
        -4,
        -2,
        10,
        1006,
        10,
        598,
        21101,
        0,
        0,
        -1,
        22202,
        -2,
        -1,
        -2,
        2107,
        0,
        -3,
        10,
        1006,
        10,
        620,
        22101,
        0,
        -1,
        1,
        21102,
        620,
        1,
        0,
        106,
        0,
        495,
        21202,
        -2,
        -1,
        -2,
        22201,
        -4,
        -2,
        -4,
        109,
        -5,
        2105,
        1,
        0,
    ];
    let mut program = IntcodeProgram::new(code.clone());

    let mut direction = Direction::North;
    let mut pos = (0, 0);
    let mut hull = HashMap::new();

    // we start with a black panel
    let mut input = vec![starting_color as i64];
    let mut output = Vec::new();
    let mut status = program.run(&mut input, &mut output);
    while status != IntcodeResult::TERMINATED && status != IntcodeResult::EOF {
        let color = match output[0] {
            0 => Color::Black,
            1 => Color::White,
            _ => panic!("Unsupported color"),
        };
        let panel = hull.entry(pos).or_insert(Panel::new());
        panel.color = color;
        panel.visited += 1;

        let turn = match output[1] {
            0 => Turn::Left,
            1 => Turn::Right,
            _ => panic!("Unsupported direction"),
        };
        direction = update_direction(direction, turn);
        pos = match direction {
            Direction::North => (pos.0, pos.1 - 1),
            Direction::South => (pos.0, pos.1 + 1),
            Direction::East => (pos.0 + 1, pos.1),
            Direction::West => (pos.0 - 1, pos.1),
        };
        debug!("direction: {:?}, pos: {:?}", direction, pos);

        output.clear();
        input.clear();
        input.push(match hull.get(&pos) {
            Some(p) => p.color as i64,
            None => Color::Black as i64,
        });
        status = program.run(&mut input, &mut output);
    }
    return hull;
}

fn part_one() -> usize {
    let hull = paint(Color::Black);
    return hull.keys().len();
}

#[test]
fn part_one_test() {
    assert_eq!(2268, part_one());
}

fn main() {
    env_logger::init();

    let part1 = part_one();
    println!("Part One: {}", part1);

    let hull = paint(Color::White);
    let mut xmin = 0;
    let mut xmax = 0;
    let mut ymin = 0;
    let mut ymax = 0;
    for k in hull.keys() {
        let kx = k.0;
        let ky = k.1;
        if kx > xmax {
            xmax = kx;
        }
        if kx < xmin {
            xmin = kx;
        }
        if ky > ymax {
            ymax = ky;
        }
        if ky < ymin {
            ymin = ky;
        }
    }
    let deltax = if xmin < 0 { -xmin } else { 0 };
    let deltay = if ymin < 0 { -ymin } else { 0 };
    info!(
        "xmin: {}, xmax: {}, deltax: {}, ymin: {}, ymax:{}, deltay: {}",
        xmin, xmax, deltax, ymin, ymax, deltay
    );

    xmax += deltax;
    ymax += deltay;

    let mut canvas: Vec<Vec<Color>> = Vec::with_capacity(ymax as usize + 1);
    for _ in 0..=ymax as usize {
        let mut row = Vec::with_capacity(xmax as usize + 1);
        for _ in 0..=xmax as usize {
            row.push(Color::Black);
        }
        canvas.push(row);
    }

    for (k, v) in hull.iter() {
        canvas[(k.1 + deltay) as usize][(k.0 + deltax) as usize] = v.color;
    }

    println!("Part 2:\n");
    // CEPKZJCR
    for row in canvas.iter() {
        for color in row.iter() {
            match color {
                Color::Black => print!(" "),
                Color::White => print!("X"),
            }
        }
        print!("\n");
    }
}