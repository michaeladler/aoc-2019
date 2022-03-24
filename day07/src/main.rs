#[macro_use]
extern crate log;

use std::collections::HashSet;

#[derive(Debug, PartialEq)]
enum OpcodeResult {
    /// Program needs more numbers to continue (number source "exhausted")
    SUSPENDED,
    /// Program terminated
    TERMINATED,
    /// EOF reached, but program did not send TERMINATE instruction
    EOF,
}

#[derive(Debug, PartialEq, Clone)]
struct OpcodeProgram<'a> {
    name: &'a str,
    code: Vec<i64>,
    ip: usize,
}

fn run_program(
    program: &mut OpcodeProgram,
    number_src: &mut Vec<i64>,
    number_target: &mut Vec<i64>,
) -> OpcodeResult {
    let n = program.code.len();
    debug!("Running program: {:?}", program);
    while program.ip < n {
        trace!("Program: {:?}, position: {}", program, program.ip);

        let mut instruction = program.code[program.ip];
        let op_code = instruction % 100;
        instruction -= op_code;
        instruction = instruction / 100;
        let param_mode1 = instruction % 10;
        instruction -= param_mode1;
        instruction = instruction / 10;
        let param_mode2 = instruction % 10;
        instruction -= param_mode2;
        instruction = instruction / 10;
        let param_mode3 = instruction % 10;

        let param_modes = [param_mode1, param_mode2, param_mode3];
        let get_param = |offset: usize| {
            let val = &program.code[program.ip + offset];
            match param_modes[offset - 1] {
                0 => &program.code[*val as usize],
                1 => val,
                _ => panic!("Unsupported param_mode"),
            }
        };

        debug!(
            "[i={}]: op_code={}, param_mode1: {}, param_mode2: {}, param_mode3: {}",
            program.ip, op_code, param_mode1, param_mode2, param_mode3
        );
        match op_code {
            // add
            1 => {
                let a = get_param(1);
                let b = get_param(2);
                let out_pos = program.code[program.ip + 3] as usize;
                debug!("[Add] param1={}, param2={}, param3={}", a, b, out_pos);
                debug!(
                    "Adding numbers {}, {} and storing result in {}",
                    a, b, out_pos
                );
                program.code[out_pos] = a + b;
                program.ip += 4;
            }
            // multiply
            2 => {
                let a = get_param(1);
                let b = get_param(2);
                let out_pos = program.code[program.ip + 3] as usize;
                debug!("[Mul] param1={}, param2={}, param3={}", a, b, out_pos);
                debug!(
                    "Multiplying numbers {}, {} and storing result in {}",
                    a, b, out_pos
                );
                program.code[out_pos] = a * b;
                program.ip += 4;
            }
            // get number
            3 => {
                let out_pos = program.code[program.ip + 1] as usize;
                debug!(
                    "[Get] Getting number and storing it in position {}",
                    out_pos
                );
                match number_src.pop() {
                    Some(n) => {
                        debug!("[Get] Received number: {}", n);
                        program.code[out_pos] = n;
                        program.ip += 2;
                    }
                    None => {
                        debug!("[Get] Need more numbers to continue");
                        return OpcodeResult::SUSPENDED;
                    }
                }
            }
            // put number
            4 => {
                let pos = program.code[program.ip + 1] as usize;
                debug!(
                    "[Put] Reading number from position {} and invoking put_number callback",
                    pos
                );
                number_target.push(program.code[pos]);
                program.ip += 2;
            }
            // jump-if-true
            5 => {
                let param1 = get_param(1);
                let param2 = get_param(2);
                debug!("[JUMP-IF-TRUE] param1={}, param2={}", param1, param2);
                program.ip = match param1 {
                    0 => {
                        debug!("No jump");
                        program.ip + 3
                    }
                    _ => {
                        debug!("Jumping");
                        *param2 as usize
                    }
                };
            }
            // jump-if-false
            6 => {
                let param1 = get_param(1);
                let param2 = get_param(2);
                debug!("[JUMP-IF-FALSE] param1={}, param2={}", param1, param2);
                program.ip = match param1 {
                    0 => {
                        debug!("Jumping");
                        *param2 as usize
                    }
                    _ => {
                        debug!("No jump");
                        program.ip + 3
                    }
                };
            }
            // less-than
            7 => {
                let a = get_param(1);
                let b = get_param(2);
                let out_pos = program.code[program.ip + 3] as usize;
                debug!(
                    "[LT] Checking if {} < {} and storing result in {}",
                    a, b, out_pos
                );
                program.code[out_pos] = if a < b { 1 } else { 0 };
                program.ip += 4;
            }
            // equals
            8 => {
                let a = get_param(1);
                let b = get_param(2);
                let out_pos = program.code[program.ip + 3] as usize;
                debug!(
                    "[EQ] Checking if {} == {} and storing result in {}",
                    a, b, out_pos
                );
                program.code[out_pos] = if a == b { 1 } else { 0 };
                program.ip += 4;
            }
            99 => {
                debug!("HALT instruction");
                return OpcodeResult::TERMINATED;
            }
            _ => panic!("Unsupported op code: {}", op_code),
        }
    }
    return OpcodeResult::EOF;
}

fn run_amplifier(program: &mut OpcodeProgram, stack: &mut Vec<i64>) -> (OpcodeResult, Option<i64>) {
    let mut result = Vec::new();
    let status = run_program(program, stack, &mut result);
    return (status, result.pop());
}

fn find_max_thruster_signal(program: &OpcodeProgram) -> i64 {
    let mut max_output = std::i64::MIN;
    let mut phases = Vec::new();
    for a in 0..=4 {
        for b in 0..=4 {
            for c in 0..=4 {
                for d in 0..=4 {
                    for e in 0..=4 {
                        let mut set = HashSet::new();
                        set.insert(a);
                        set.insert(b);
                        set.insert(c);
                        set.insert(d);
                        set.insert(e);
                        if set.len() < 5 {
                            continue;
                        }

                        debug!(
                            "Testing amplifier configuration: {}, {}, {}, {}, {}",
                            a, b, c, d, e
                        );
                        let (_, output) = run_amplifier(&mut program.clone(), &mut vec![0, a]);
                        let output = output.expect("Program did not return any output");
                        let (_, output) = run_amplifier(&mut program.clone(), &mut vec![output, b]);
                        let output = output.expect("Program did not return any output");
                        let (_, output) = run_amplifier(&mut program.clone(), &mut vec![output, c]);
                        let output = output.expect("Program did not return any output");
                        let (_, output) = run_amplifier(&mut program.clone(), &mut vec![output, d]);
                        let output = output.expect("Program did not return any output");
                        let (_, output) = run_amplifier(&mut program.clone(), &mut vec![output, e]);
                        let output = output.expect("Program did not return any output");
                        if output > max_output {
                            debug!("Found new maximum! Phases: {:?}", phases);
                            max_output = output;
                            phases = vec![a, b, c, d, e];
                        }
                    }
                }
            }
        }
    }
    debug!("Phases: {:?}", phases);
    max_output
}

fn find_max_thruster_signal_feedback_loop(program: &OpcodeProgram) -> i64 {
    let mut max_output = std::i64::MIN;
    for a in 5..=9 {
        for b in 5..=9 {
            for c in 5..=9 {
                for d in 5..=9 {
                    for e in 5..=9 {
                        // find duplicate numbers
                        let mut set = HashSet::new();
                        set.insert(a);
                        set.insert(b);
                        set.insert(c);
                        set.insert(d);
                        set.insert(e);
                        if set.len() < 5 {
                            continue;
                        }
                        // numbers are distinct from this point forward
                        debug!("Testing phase settings: {},{},{},{},{}", a, b, c, d, e);

                        let mut program_a = program.clone();
                        program_a.name = "A";
                        let mut program_b = program.clone();
                        program_b.name = "B";
                        let mut program_c = program.clone();
                        program_c.name = "C";
                        let mut program_d = program.clone();
                        program_d.name = "D";
                        let mut program_e = program.clone();
                        program_e.name = "E";

                        let mut stack = Vec::new();
                        stack.push(0);
                        stack.push(a);

                        // First loop / initialization
                        debug!("Amplifier initialization (first run)");
                        let (status, output) = run_amplifier(&mut program_a, &mut stack);
                        let output = output.expect("Program A did not return any output");
                        debug!("status: {:?}, output: {}", status, output);

                        stack.clear();
                        stack.push(output);
                        stack.push(b);
                        let (status, output) = run_amplifier(&mut program_b, &mut stack);
                        let output = output.expect("Program B did not return any output");
                        debug!("status: {:?}, output: {}", status, output);

                        stack.clear();
                        stack.push(output);
                        stack.push(c);
                        let (status, output) = run_amplifier(&mut program_c, &mut stack);
                        let output = output.expect("Program C did not return any output");
                        debug!("status: {:?}, output: {}", status, output);

                        stack.clear();
                        stack.push(output);
                        stack.push(d);
                        let (status, output) = run_amplifier(&mut program_d, &mut stack);
                        let output = output.expect("Program D did not return any output");
                        debug!("status: {:?}, output: {}", status, output);

                        stack.clear();
                        stack.push(output);
                        stack.push(e);
                        let (status, output) = run_amplifier(&mut program_e, &mut stack);
                        let output = output.expect("Program E did not return any output");
                        debug!("status: {:?}, output: {}", status, output);

                        stack.clear();
                        stack.push(output);
                        debug!("Starting feedback loop");
                        loop {
                            let (status, output) = run_amplifier(&mut program_a, &mut stack);
                            let output = output.expect("Program A did not return any output");
                            stack.clear();
                            stack.push(output);
                            debug!("status: {:?}, output: {}", status, output);

                            let (status, output) = run_amplifier(&mut program_b, &mut stack);
                            let output = output.expect("Program B did not return any output");
                            stack.clear();
                            stack.push(output);
                            debug!("status: {:?}, output: {}", status, output);

                            let (status, output) = run_amplifier(&mut program_c, &mut stack);
                            let output = output.expect("Program C did not return any output");
                            stack.clear();
                            stack.push(output);
                            debug!("status: {:?}, output: {}", status, output);

                            let (status, output) = run_amplifier(&mut program_d, &mut stack);
                            let output = output.expect("Program D did not return any output");
                            stack.clear();
                            stack.push(output);
                            debug!("status: {:?}, output: {}", status, output);

                            let (status, output) = run_amplifier(&mut program_e, &mut stack);
                            let output = output.expect("Program E did not return any output");
                            stack.clear();
                            stack.push(output);
                            debug!("status: {:?}, output: {}", status, output);

                            if status == OpcodeResult::TERMINATED {
                                debug!("Amplifier E terminated");
                                if output > max_output {
                                    debug!("new maximum is {}, was: {}", output, max_output);
                                    max_output = output;
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
    max_output
}

fn solve_problem() -> (i64, i64) {
    let code = vec![
        3, 8, 1001, 8, 10, 8, 105, 1, 0, 0, 21, 42, 67, 88, 105, 114, 195, 276, 357, 438, 99999, 3,
        9, 101, 4, 9, 9, 102, 3, 9, 9, 1001, 9, 2, 9, 102, 4, 9, 9, 4, 9, 99, 3, 9, 1001, 9, 4, 9,
        102, 4, 9, 9, 101, 2, 9, 9, 1002, 9, 5, 9, 1001, 9, 2, 9, 4, 9, 99, 3, 9, 1001, 9, 4, 9,
        1002, 9, 4, 9, 101, 2, 9, 9, 1002, 9, 2, 9, 4, 9, 99, 3, 9, 101, 4, 9, 9, 102, 3, 9, 9,
        1001, 9, 5, 9, 4, 9, 99, 3, 9, 102, 5, 9, 9, 4, 9, 99, 3, 9, 102, 2, 9, 9, 4, 9, 3, 9, 101,
        1, 9, 9, 4, 9, 3, 9, 101, 2, 9, 9, 4, 9, 3, 9, 1001, 9, 2, 9, 4, 9, 3, 9, 102, 2, 9, 9, 4,
        9, 3, 9, 1002, 9, 2, 9, 4, 9, 3, 9, 1001, 9, 1, 9, 4, 9, 3, 9, 102, 2, 9, 9, 4, 9, 3, 9,
        102, 2, 9, 9, 4, 9, 3, 9, 101, 2, 9, 9, 4, 9, 99, 3, 9, 1002, 9, 2, 9, 4, 9, 3, 9, 1001, 9,
        2, 9, 4, 9, 3, 9, 102, 2, 9, 9, 4, 9, 3, 9, 102, 2, 9, 9, 4, 9, 3, 9, 102, 2, 9, 9, 4, 9,
        3, 9, 102, 2, 9, 9, 4, 9, 3, 9, 1001, 9, 1, 9, 4, 9, 3, 9, 101, 1, 9, 9, 4, 9, 3, 9, 101,
        1, 9, 9, 4, 9, 3, 9, 1002, 9, 2, 9, 4, 9, 99, 3, 9, 102, 2, 9, 9, 4, 9, 3, 9, 101, 1, 9, 9,
        4, 9, 3, 9, 1002, 9, 2, 9, 4, 9, 3, 9, 1002, 9, 2, 9, 4, 9, 3, 9, 1002, 9, 2, 9, 4, 9, 3,
        9, 102, 2, 9, 9, 4, 9, 3, 9, 101, 2, 9, 9, 4, 9, 3, 9, 102, 2, 9, 9, 4, 9, 3, 9, 1001, 9,
        2, 9, 4, 9, 3, 9, 1001, 9, 1, 9, 4, 9, 99, 3, 9, 1002, 9, 2, 9, 4, 9, 3, 9, 1001, 9, 1, 9,
        4, 9, 3, 9, 101, 2, 9, 9, 4, 9, 3, 9, 1001, 9, 1, 9, 4, 9, 3, 9, 101, 2, 9, 9, 4, 9, 3, 9,
        1001, 9, 1, 9, 4, 9, 3, 9, 102, 2, 9, 9, 4, 9, 3, 9, 1002, 9, 2, 9, 4, 9, 3, 9, 1002, 9, 2,
        9, 4, 9, 3, 9, 1001, 9, 2, 9, 4, 9, 99, 3, 9, 1002, 9, 2, 9, 4, 9, 3, 9, 101, 1, 9, 9, 4,
        9, 3, 9, 1002, 9, 2, 9, 4, 9, 3, 9, 102, 2, 9, 9, 4, 9, 3, 9, 1001, 9, 1, 9, 4, 9, 3, 9,
        1002, 9, 2, 9, 4, 9, 3, 9, 1001, 9, 1, 9, 4, 9, 3, 9, 101, 2, 9, 9, 4, 9, 3, 9, 1001, 9, 1,
        9, 4, 9, 3, 9, 1002, 9, 2, 9, 4, 9, 99,
    ];
    let program = OpcodeProgram {
        name: "Input",
        code,
        ip: 0,
    };

    let part1 = find_max_thruster_signal(&program);
    let part2 = find_max_thruster_signal_feedback_loop(&program);
    return (part1, part2);
}

fn main() {
    env_logger::init();
    let (part1, part2) = solve_problem();
    println!("Part one: {}", part1);
    println!("Part two: {}", part2);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn solve_problem_test() {
        init();
        let (part1, part2) = solve_problem();
        assert_eq!(212460, part1);
        assert_eq!(21844737, part2);
    }

    #[test]
    fn test_run_program() {
        init();
        let mut result = Vec::new();

        let code = vec![1, 0, 0, 0, 99];
        let mut prog = OpcodeProgram {
            name: "test",
            code,
            ip: 0,
        };
        run_program(&mut prog, &mut vec![42], &mut result);
        assert_eq!(prog.code, [2, 0, 0, 0, 99]);

        let code = vec![2, 3, 0, 3, 99];
        let mut prog = OpcodeProgram {
            name: "test",
            code,
            ip: 0,
        };
        run_program(&mut prog, &mut vec![42], &mut result);
        assert_eq!(prog.code, [2, 3, 0, 6, 99]);

        let code = vec![2, 4, 4, 5, 99, 0];
        let mut prog = OpcodeProgram {
            name: "test",
            code,
            ip: 0,
        };
        run_program(&mut prog, &mut vec![42], &mut result);
        assert_eq!(prog.code, [2, 4, 4, 5, 99, 9801]);

        let code = vec![1, 1, 1, 4, 99, 5, 6, 0, 99];
        let mut prog = OpcodeProgram {
            name: "test",
            code,
            ip: 0,
        };
        run_program(&mut prog, &mut vec![42], &mut result);
        assert_eq!(prog.code, [30, 1, 1, 4, 2, 5, 6, 0, 99]);
    }

    #[test]
    fn test_run_program_parameter_mode() {
        init();
        let mut result = Vec::new();

        let code = vec![1002, 4, 3, 4, 33];
        let mut prog = OpcodeProgram {
            name: "test",
            code,
            ip: 0,
        };
        run_program(&mut prog, &mut vec![42], &mut result);
        assert_eq!(prog.code, [1002, 4, 3, 4, 99]);
    }

    #[test]
    fn test_jump_if_equal_position_mode() {
        init();
        let mut _output: Vec<i64> = Vec::new();

        // Using position mode, consider whether the input is equal to 8; output 1 (if it is) or 0 (if it is not).
        let code = vec![3, 9, 8, 9, 10, 9, 4, 9, 99, -1, 8];
        let program = OpcodeProgram {
            name: "test",
            code,
            ip: 0,
        };

        let mut output = Vec::new();
        run_program(&mut program.clone(), &mut vec![8], &mut output);
        assert_eq!(output, vec![1]);

        output.clear();
        run_program(&mut program.clone(), &mut vec![7], &mut output);
        assert_eq!(output, vec![0]);
    }

    #[test]
    fn test_jump_if_less_than_position_mode() {
        init();
        let mut _output: Vec<i64> = Vec::new();

        // Using position mode, consider whether the input is less than 8; output 1 (if it is) or 0 (if it is not).
        let code = vec![3, 9, 7, 9, 10, 9, 4, 9, 99, -1, 8];
        let program = OpcodeProgram {
            name: "test",
            code,
            ip: 0,
        };

        let mut output = Vec::new();
        run_program(&mut program.clone(), &mut vec![7], &mut output);
        assert_eq!(output, vec![1]);

        output.clear();
        run_program(&mut program.clone(), &mut vec![8], &mut output);
        assert_eq!(output, vec![0]);
    }

    #[test]
    fn test_jump_if_equal_immediate_mode() {
        init();
        let mut _output: Vec<i64> = Vec::new();

        // Using immediate mode, consider whether the input is equal to 8; output 1 (if it is) or 0 (if it is not)
        let code = vec![3, 3, 1108, -1, 8, 3, 4, 3, 99];
        let program = OpcodeProgram {
            name: "test",
            code,
            ip: 0,
        };

        let mut output = Vec::new();
        run_program(&mut program.clone(), &mut vec![8], &mut output);
        assert_eq!(output, vec![1]);

        output.clear();
        run_program(&mut program.clone(), &mut vec![7], &mut output);
        assert_eq!(output, vec![0]);
    }

    #[test]
    fn test_jump_if_less_than_immediate_mode() {
        init();
        let mut _output: Vec<i64> = Vec::new();

        //  Using immediate mode, consider whether the input is less than 8; output 1 (if it is) or 0 (if it is not).
        let code = vec![3, 3, 1107, -1, 8, 3, 4, 3, 99];
        let program = OpcodeProgram {
            name: "test",
            code,
            ip: 0,
        };

        let mut output = Vec::new();
        run_program(&mut program.clone(), &mut vec![7], &mut output);
        assert_eq!(output, vec![1]);

        output.clear();
        run_program(&mut program.clone(), &mut vec![8], &mut output);
        assert_eq!(output, vec![0]);
    }

    #[test]
    fn test_jumps_position_mode() {
        init();
        let mut _output: Vec<i64> = Vec::new();

        // Here are some jump tests that take an input, then output 0 if the input was zero or 1 if the input was non-zero:
        let code = vec![3, 12, 6, 12, 15, 1, 13, 14, 13, 4, 13, 99, -1, 0, 1, 9];
        let program = OpcodeProgram {
            name: "test",
            code,
            ip: 0,
        };

        let mut output = Vec::new();
        run_program(&mut program.clone(), &mut vec![0], &mut output);
        assert_eq!(output, vec![0]);

        output.clear();
        run_program(&mut program.clone(), &mut vec![2], &mut output);
        assert_eq!(output, vec![1]);
    }

    #[test]
    fn test_jumps_immediate_mode() {
        init();
        let mut _output: Vec<i64> = Vec::new();

        // Here are some jump tests that take an input, then output 0 if the input was zero or 1 if the input was non-zero:
        let code = vec![3, 3, 1105, -1, 9, 1101, 0, 0, 12, 4, 12, 99, 1];
        let program = OpcodeProgram {
            name: "test",
            code,
            ip: 0,
        };

        let mut output = Vec::new();
        run_program(&mut program.clone(), &mut vec![0], &mut output);
        assert_eq!(output, vec![0]);

        output.clear();
        run_program(&mut program.clone(), &mut vec![2], &mut output);
        assert_eq!(output, vec![1]);
    }

    #[test]
    fn test_find_max_thruster_signal() {
        let code = vec![
            3, 15, 3, 16, 1002, 16, 10, 16, 1, 16, 15, 15, 4, 15, 99, 0, 0,
        ];
        let program = OpcodeProgram {
            name: "test",
            code,
            ip: 0,
        };
        assert_eq!(find_max_thruster_signal(&program), 43210);

        let code = vec![
            3, 23, 3, 24, 1002, 24, 10, 24, 1002, 23, -1, 23, 101, 5, 23, 23, 1, 24, 23, 23, 4, 23,
            99, 0, 0,
        ];
        let program = OpcodeProgram {
            name: "test",
            code,
            ip: 0,
        };
        assert_eq!(find_max_thruster_signal(&program), 54321);

        let code = vec![
            3, 31, 3, 32, 1002, 32, 10, 32, 1001, 31, -2, 31, 1007, 31, 0, 33, 1002, 33, 7, 33, 1,
            33, 31, 31, 1, 32, 31, 31, 4, 31, 99, 0, 0, 0,
        ];
        let program = OpcodeProgram {
            name: "test",
            code,
            ip: 0,
        };
        assert_eq!(find_max_thruster_signal(&program), 65210);
    }

    #[test]
    fn test_find_max_thruster_signal_feedback_loop() {
        let code = vec![
            3, 26, 1001, 26, -4, 26, 3, 27, 1002, 27, 2, 27, 1, 27, 26, 27, 4, 27, 1001, 28, -1,
            28, 1005, 28, 6, 99, 0, 0, 5,
        ];
        let program = OpcodeProgram {
            name: "test",
            code,
            ip: 0,
        };
        assert_eq!(find_max_thruster_signal_feedback_loop(&program), 139629729);

        let code = vec![
            3, 52, 1001, 52, -5, 52, 3, 53, 1, 52, 56, 54, 1007, 54, 5, 55, 1005, 55, 26, 1001, 54,
            -5, 54, 1105, 1, 12, 1, 53, 54, 53, 1008, 54, 0, 55, 1001, 55, 1, 55, 2, 53, 55, 53, 4,
            53, 1001, 56, -1, 56, 1005, 56, 6, 99, 0, 0, 0, 0, 10,
        ];
        let program = OpcodeProgram {
            name: "test",
            code,
            ip: 0,
        };
        assert_eq!(find_max_thruster_signal_feedback_loop(&program), 18216);
    }
}
