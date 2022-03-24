use crate::springscript::{Instruction, Springscript, RR, RW};
use aoc2019::intcode::IntcodeProgram;

use Instruction::*;

fn run(prog: &mut IntcodeProgram, input: &[i64]) -> Option<i64> {
    let mut output = Vec::new();
    prog.run(input, &mut output);

    // scan for non-ascii
    if let Some(answer) = output.iter().find(|&&x| x >= 128) {
        return Some(*answer);
    }
    for &c in output.iter() {
        eprint!("{}", (c as u8) as char);
    }
    return None;
}

fn walk_droid(prog: &mut IntcodeProgram, code: &Springscript) -> Option<i64> {
    let mut input = code.encode();
    for &c in &['W', 'A', 'L', 'K'] {
        input.push((c as u8) as i64);
    }
    input.push(10); // newline
    run(prog, &input)
}

fn run_droid(prog: &mut IntcodeProgram, code: &Springscript) -> Option<i64> {
    let mut input = code.encode();
    for &c in &['R', 'U', 'N'] {
        input.push((c as u8) as i64);
    }
    input.push(10); // newline
    run(prog, &input)
}

pub fn part1(fname: &str) -> i64 {
    // brute-force won't work: 36^15 = 2.2 * 10^23
    // try back-tracking approach?
    // There are only TWO writeable registers (hopefully will not change in part 2).
    // Rules:
    //      (4) First instruction != AND
    //      (1) NOT X Y must not be followed by NOT X' Y
    //      (2) AND X Y must not be followed by AND Y X         symmetry
    //      (6) AND X Y must not be followed by NOT _ Y
    //      (3) OR X Y must not be followed by OR Y X           symmetry
    //      (5) OR X Y must not be followed by NOT _ Y
    //
    let mut prog = IntcodeProgram::from_file(fname).unwrap();
    // If there is ground at the given distance, the register will be true;
    // if there is a hole, the register will be false.
    //
    // A jump is 4 tiles, i.e. new_pos = current_pos + 4
    //
    // Greedy Algo: try to jump as early as possible
    #[rustfmt::skip]
    let code = Springscript::new(vec![
        // tile at D AND no tile at C => JUMP
        // ###J#..#.#######
        OR(RR::D, RW::T),  // tile at D?
        NOT(RR::C, RW::J), // no tile at C
        AND(RR::T, RW::J),

        // no tile at A => JUMP
        // #####..J.#######
        NOT(RR::A, RW::T), // no tile at A
        OR(RR::T, RW::J),
    ]);
    walk_droid(&mut prog, &code).expect("no solution found")
}

pub fn part2(fname: &str) -> i64 {
    let mut prog = IntcodeProgram::from_file(fname).unwrap();

    // ABCDEFGHI
    // 123456789

    #[rustfmt::skip]
    let code = Springscript::new(vec![
        NOT(RR::A, RW::J), // no tile at A

        NOT(RR::B, RW::T),
        OR(RR::T, RW::J), // no tile at A or no tile at B

        NOT(RR::C, RW::T),
        OR(RR::T, RW::J), // no tile at A or no tile at B or no tile at C

        AND(RR::D, RW::J), // (no tile at A or no tile at B or no tile at C) AND (tile at D)

        // copy H to T
        NOT(RR::H, RW::T),
        NOT(RR::T, RW::T),

        OR(RR::E, RW::T), // tile at E or H
        AND(RR::T, RW::J), // (no tile at A or no tile at B or no tile at C) AND (tile at D) AND (tile at E or H)
    ]);

    run_droid(&mut prog, &code).expect("no solution found")
}
