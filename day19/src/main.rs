use std::env;
use std::fmt;
use std::time::Instant;

use env_logger;
use log::{debug, info, trace};

use aoc2019::intcode::IntcodeProgram;

const PULLED: u8 = 1;

#[derive(Debug)]
struct Grid {
    tiles: Vec<Row>,
}

impl Grid {
    pub fn new() -> Self {
        Self { tiles: Vec::new() }
    }
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let n = self.tiles.len();
        for y in 0..n {
            let offset = self.tiles[y].offset;
            let count = self.tiles[y].count;
            for x in 0..=offset + count {
                if x >= offset && x < offset + count {
                    write!(f, "#")?;
                } else {
                    write!(f, ".")?;
                }
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

#[derive(Debug)]
struct Row {
    offset: usize,
    count: usize,
}

struct TractorBeam {
    prog: IntcodeProgram,
    grid: Grid,
}

impl TractorBeam {
    pub fn new(prog: IntcodeProgram) -> Self {
        let mut grid = Grid::new();
        grid.tiles.push(Row {
            offset: 0,
            count: 1,
        });
        Self { prog, grid }
    }

    /// calculates the next row of the tractor beam; some rows are empty
    /// though.
    pub fn next_row(&mut self) {
        let start_y = self.grid.tiles.len();

        let mut start_x = self.grid.tiles[start_y - 1].offset;
        //we use max_col to determine how far to the right we shall attempt to
        //search.
        let mut max_col = start_x + self.grid.tiles[start_y - 1].count + 3;

        // if last row was empty, go north until we find a non-empty one
        if start_x == 0 && self.grid.tiles[start_y - 1].count == 0 {
            debug!("last row is empty");
            for i in (0..start_y - 1).rev() {
                debug!("i: {}", i);
                let count = self.grid.tiles[i].count;
                if count > 0 {
                    start_x = self.grid.tiles[i].offset;
                    max_col = start_x + self.grid.tiles[i].count + 2;
                    break;
                }
            }
        }
        debug!(
            "start_y: {}, start_x: {}, max_col: {}",
            start_y, start_x, max_col
        );

        // make it immutable
        let max_col = max_col;

        let mut number_src = [0; 2];
        number_src[1] = start_y as i64;

        let mut output = Vec::new();
        let mut pulled_counter = 0;

        let mut found_first_pulled = false;
        for x in start_x..usize::MAX {
            number_src[0] = x as i64;
            output.clear();
            self.prog.clone().run(&number_src, &mut output);
            let is_pulled = output[0] as u8 == PULLED;
            if found_first_pulled {
                if is_pulled {
                    pulled_counter += 1;
                } else {
                    // we are done with this row
                    break;
                }
            } else {
                if is_pulled {
                    pulled_counter += 1;
                    found_first_pulled = true;
                    start_x = x;
                }
                // what if there is no pulled tile in this row?
                if x >= max_col {
                    // give up
                    break;
                }
            }
        }

        trace!("row {} has pulled count: {}", start_y, pulled_counter);
        self.grid.tiles.push(Row {
            offset: start_x,
            count: pulled_counter,
        });
    }
}

fn part1(fname: &str) -> usize {
    let prog = IntcodeProgram::from_file(fname).unwrap();
    let mut tractor = TractorBeam::new(prog);
    let n = 50;
    for _ in 1..n {
        tractor.next_row();
    }
    debug!("\n{}", tractor.grid);
    let mut count = 0;
    for row in tractor.grid.tiles {
        if row.offset + row.count < n {
            count += row.count;
        } else {
            // cut off
            if row.offset < n {
                count += n - row.offset;
            }
        }
    }
    return count;
}

fn part2(fname: &str) -> usize {
    let prog = IntcodeProgram::from_file(fname).unwrap();
    let mut tractor = TractorBeam::new(prog);
    // Idea:
    // The south-west point of the rectangle must be the *starting* point (#)
    // of a row; otherwise, the rectangle would not have minimal distance from
    // the origin (we could move the rectangle west then).
    // So we try to build squares of size 100x100 with starting point SE.
    const RECT_SIZE: usize = 100;
    for _ in 1..RECT_SIZE {
        tractor.next_row();
    }
    let mut best_x = 0;
    let mut best_y = 0;
    'outer: for y in RECT_SIZE - 1..usize::MAX {
        tractor.next_row();
        let row = &tractor.grid.tiles[y];
        if row.count < RECT_SIZE {
            continue 'outer;
        }
        let top_idx = y + 1 - RECT_SIZE;
        let offset = row.offset;
        info!(
            "row {}, col {} east ok, trying to go north to row {}",
            y, offset, top_idx
        );
        best_y = top_idx;
        best_x = offset;

        // check if we can walk up to top_row
        for i in (top_idx..y).rev() {
            // adjust count, because it could be that offset < y
            let mut count = tractor.grid.tiles[i].count;
            if tractor.grid.tiles[i].offset < offset {
                let delta = offset - tractor.grid.tiles[i].offset;
                count -= delta;
            }
            let count = count;
            debug!("row {}, col {} has {} tiles to the right", i, offset, count);
            if count < RECT_SIZE {
                info!(
                    "cannot go north, row {}, col {} does not have enough tiles",
                    i, offset
                );
                continue 'outer;
            }
        }
        break 'outer;
    }
    info!("top-left point: row {}, col {}", best_y, best_x);
    return best_x * 10000 + best_y;
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
    fn part1_test() {
        assert_eq!(131, part1(&"input.txt"[..]));
    }

    #[test]
    fn part2_test() {
        assert_eq!(15231022, part2(&"input.txt"[..]));
    }
}
