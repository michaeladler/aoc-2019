use log::debug;

fn run_program(
    program: &mut [i64],
    get_number: impl Fn() -> i64,
    mut put_number: impl FnMut(i64) -> (),
) {
    let n = program.len();
    let mut i = 0;
    'outer: while i < n {
        debug!("Program: {:?}", program);

        let mut instruction = program[i];
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
            let val = &program[i + offset];
            match param_modes[offset - 1] {
                0 => &program[*val as usize],
                1 => val,
                _ => panic!("Unsupported param_mode"),
            }
        };

        debug!(
            "[i={}]: op_code={}, param_mode1: {}, param_mode2: {}, param_mode3: {}",
            i, op_code, param_mode1, param_mode2, param_mode3
        );
        match op_code {
            // add
            1 => {
                let a = get_param(1);
                let b = get_param(2);
                let out_pos = program[i + 3] as usize;
                debug!("[Add] param1={}, param2={}, param3={}", a, b, out_pos);
                program[out_pos] = a + b;
                i += 4;
            }
            // multiply
            2 => {
                let a = get_param(1);
                let b = get_param(2);
                let out_pos = program[i + 3] as usize;
                debug!("[Mul] param1={}, param2={}, param3={}", a, b, out_pos);
                program[out_pos] = a * b;
                i += 4;
            }
            // get number
            3 => {
                let out_pos = program[i + 1] as usize;
                debug!("[Get] param1={}", out_pos);
                let n = get_number();
                debug!("Got: {}", n);
                program[out_pos] = n;
                i += 2;
            }
            // put number
            4 => {
                let pos = program[i + 1] as usize;
                debug!("[Put] param1={}", pos);
                put_number(program[pos]);
                i += 2;
            }
            // jump-if-true
            5 => {
                let param1 = get_param(1);
                let param2 = get_param(2);
                debug!("[JUMP-IF-TRUE] param1={}, param2={}", param1, param2);
                i = match param1 {
                    0 => i + 3,
                    _ => *param2 as usize,
                };
            }
            // jump-if-false
            6 => {
                let param1 = get_param(1);
                let param2 = get_param(2);
                debug!("[JUMP-IF-FALSE] param1={}, param2={}", param1, param2);
                i = match param1 {
                    0 => *param2 as usize,
                    _ => i + 3,
                };
            }
            // less-than
            7 => {
                let a = get_param(1);
                let b = get_param(2);
                let out_pos = program[i + 3] as usize;
                debug!("[LT] param1={}, param2={}, param3={}", a, b, out_pos);
                program[out_pos] = if get_param(1) < get_param(2) { 1 } else { 0 };
                i += 4;
            }
            // equals
            8 => {
                let a = get_param(1);
                let b = get_param(2);
                let out_pos = program[i + 3] as usize;
                debug!("[EQ] param1={}, param2={}, param3={}", a, b, out_pos);
                program[out_pos] = if get_param(1) == get_param(2) { 1 } else { 0 };
                i += 4;
            }
            99 => {
                debug!("EOF");
                break 'outer;
            }
            _ => panic!("Unsupported op code: {}", op_code),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_run_program() {
        init();

        let mut prog = [1, 0, 0, 0, 99];
        run_program(&mut prog, || 42, |_| {});
        assert_eq!(prog, [2, 0, 0, 0, 99]);

        let mut prog = [2, 3, 0, 3, 99];
        run_program(&mut prog, || 42, |_| {});
        assert_eq!(prog, [2, 3, 0, 6, 99]);

        let mut prog = [2, 4, 4, 5, 99, 0];
        run_program(&mut prog, || 42, |_| {});
        assert_eq!(prog, [2, 4, 4, 5, 99, 9801]);

        let mut prog = [1, 1, 1, 4, 99, 5, 6, 0, 99];
        run_program(&mut prog, || 42, |_| {});
        assert_eq!(prog, [30, 1, 1, 4, 2, 5, 6, 0, 99]);
    }

    #[test]
    fn test_run_program_parameter_mode() {
        init();

        let mut prog = [1002, 4, 3, 4, 33];
        run_program(&mut prog, || 42, |_| {});
        assert_eq!(prog, [1002, 4, 3, 4, 99]);
    }

    #[test]
    fn test_jump_if_equal_position_mode() {
        init();

        // Using position mode, consider whether the input is equal to 8; output 1 (if it is) or 0 (if it is not).
        let program = [3, 9, 8, 9, 10, 9, 4, 9, 99, -1, 8];

        let mut output = Vec::new();
        run_program(
            &mut program.clone(),
            || 8,
            |a| {
                output.push(a);
            },
        );
        assert_eq!(output, vec![1]);

        output.clear();
        run_program(
            &mut program.clone(),
            || 7,
            |a| {
                output.push(a);
            },
        );
        assert_eq!(output, vec![0]);
    }

    #[test]
    fn test_jump_if_less_than_position_mode() {
        init();

        // Using position mode, consider whether the input is less than 8; output 1 (if it is) or 0 (if it is not).
        let program = [3, 9, 7, 9, 10, 9, 4, 9, 99, -1, 8];

        let mut output = Vec::new();
        run_program(
            &mut program.clone(),
            || 7,
            |a| {
                output.push(a);
            },
        );
        assert_eq!(output, vec![1]);

        output.clear();
        run_program(
            &mut program.clone(),
            || 8,
            |a| {
                output.push(a);
            },
        );
        assert_eq!(output, vec![0]);
    }

    #[test]
    fn test_jump_if_equal_immediate_mode() {
        init();

        // Using immediate mode, consider whether the input is equal to 8; output 1 (if it is) or 0 (if it is not)
        let program = [3, 3, 1108, -1, 8, 3, 4, 3, 99];

        let mut output = Vec::new();
        run_program(
            &mut program.clone(),
            || 8,
            |a| {
                output.push(a);
            },
        );
        assert_eq!(output, vec![1]);

        output.clear();
        run_program(
            &mut program.clone(),
            || 7,
            |a| {
                output.push(a);
            },
        );
        assert_eq!(output, vec![0]);
    }

    #[test]
    fn test_jump_if_less_than_immediate_mode() {
        init();

        //  Using immediate mode, consider whether the input is less than 8; output 1 (if it is) or 0 (if it is not).
        let program = [3, 3, 1107, -1, 8, 3, 4, 3, 99];

        let mut output = Vec::new();
        run_program(
            &mut program.clone(),
            || 7,
            |a| {
                output.push(a);
            },
        );
        assert_eq!(output, vec![1]);

        output.clear();
        run_program(
            &mut program.clone(),
            || 8,
            |a| {
                output.push(a);
            },
        );
        assert_eq!(output, vec![0]);
    }

    #[test]
    fn test_jumps_position_mode() {
        init();
        // Here are some jump tests that take an input, then output 0 if the input was zero or 1 if the input was non-zero:
        let program = [3, 12, 6, 12, 15, 1, 13, 14, 13, 4, 13, 99, -1, 0, 1, 9];

        let mut output = Vec::new();
        run_program(
            &mut program.clone(),
            || 0,
            |a| {
                output.push(a);
            },
        );
        assert_eq!(output, vec![0]);

        output.clear();
        run_program(
            &mut program.clone(),
            || 2,
            |a| {
                output.push(a);
            },
        );
        assert_eq!(output, vec![1]);
    }

    #[test]
    fn test_jumps_immediate_mode() {
        init();
        // Here are some jump tests that take an input, then output 0 if the input was zero or 1 if the input was non-zero:
        let program = [3, 3, 1105, -1, 9, 1101, 0, 0, 12, 4, 12, 99, 1];

        let mut output = Vec::new();
        run_program(
            &mut program.clone(),
            || 0,
            |a| {
                output.push(a);
            },
        );
        assert_eq!(output, vec![0]);

        output.clear();
        run_program(
            &mut program.clone(),
            || 2,
            |a| {
                output.push(a);
            },
        );
        assert_eq!(output, vec![1]);
    }

    #[test]
    #[ignore]
    fn test_jumps_large() {
        init();

        // This program uses an input instruction to ask for a single number. The program will then output 999 if the input value is below 8, output 1000 if the input value is equal to 8, or output 1001 if the input value is greater than 8.
        let program = [
            3, 21, 1008, 21, 8, 20, 1005, 20, 22, 107, 8, 21, 20, 1006, 20, 31, 1106, 0, 36, 98, 0,
            0, 1002, 21, 125, 20, 4, 20, 1105, 1, 46, 104, 999, 1105, 1, 46, 1101, 1000, 1, 20, 4,
            20, 1105, 1, 46, 98, 99,
        ];

        let mut output = Vec::new();
        run_program(
            &mut program.clone(),
            || 7,
            |a| {
                output.push(a);
            },
        );
        assert_eq!(output, vec![999]);

        output.clear();
        run_program(
            &mut program.clone(),
            || 8,
            |a| {
                output.push(a);
            },
        );
        assert_eq!(output, vec![1000]);

        output.clear();
        run_program(
            &mut program.clone(),
            || 9,
            |a| {
                output.push(a);
            },
        );
        assert_eq!(output, vec![1001]);
    }
}

fn main() {
    env_logger::init();

    let program = [
        3, 225, 1, 225, 6, 6, 1100, 1, 238, 225, 104, 0, 1101, 91, 67, 225, 1102, 67, 36, 225,
        1102, 21, 90, 225, 2, 13, 48, 224, 101, -819, 224, 224, 4, 224, 1002, 223, 8, 223, 101, 7,
        224, 224, 1, 223, 224, 223, 1101, 62, 9, 225, 1, 139, 22, 224, 101, -166, 224, 224, 4, 224,
        1002, 223, 8, 223, 101, 3, 224, 224, 1, 223, 224, 223, 102, 41, 195, 224, 101, -2870, 224,
        224, 4, 224, 1002, 223, 8, 223, 101, 1, 224, 224, 1, 224, 223, 223, 1101, 46, 60, 224, 101,
        -106, 224, 224, 4, 224, 1002, 223, 8, 223, 1001, 224, 2, 224, 1, 224, 223, 223, 1001, 191,
        32, 224, 101, -87, 224, 224, 4, 224, 102, 8, 223, 223, 1001, 224, 1, 224, 1, 223, 224, 223,
        1101, 76, 90, 225, 1101, 15, 58, 225, 1102, 45, 42, 224, 101, -1890, 224, 224, 4, 224,
        1002, 223, 8, 223, 1001, 224, 5, 224, 1, 224, 223, 223, 101, 62, 143, 224, 101, -77, 224,
        224, 4, 224, 1002, 223, 8, 223, 1001, 224, 4, 224, 1, 224, 223, 223, 1101, 55, 54, 225,
        1102, 70, 58, 225, 1002, 17, 80, 224, 101, -5360, 224, 224, 4, 224, 102, 8, 223, 223, 1001,
        224, 3, 224, 1, 223, 224, 223, 4, 223, 99, 0, 0, 0, 677, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        1105, 0, 99999, 1105, 227, 247, 1105, 1, 99999, 1005, 227, 99999, 1005, 0, 256, 1105, 1,
        99999, 1106, 227, 99999, 1106, 0, 265, 1105, 1, 99999, 1006, 0, 99999, 1006, 227, 274,
        1105, 1, 99999, 1105, 1, 280, 1105, 1, 99999, 1, 225, 225, 225, 1101, 294, 0, 0, 105, 1, 0,
        1105, 1, 99999, 1106, 0, 300, 1105, 1, 99999, 1, 225, 225, 225, 1101, 314, 0, 0, 106, 0, 0,
        1105, 1, 99999, 1008, 677, 677, 224, 102, 2, 223, 223, 1005, 224, 329, 1001, 223, 1, 223,
        1108, 677, 226, 224, 1002, 223, 2, 223, 1006, 224, 344, 101, 1, 223, 223, 107, 677, 226,
        224, 1002, 223, 2, 223, 1006, 224, 359, 101, 1, 223, 223, 108, 677, 677, 224, 1002, 223, 2,
        223, 1006, 224, 374, 1001, 223, 1, 223, 108, 226, 677, 224, 1002, 223, 2, 223, 1006, 224,
        389, 101, 1, 223, 223, 7, 226, 677, 224, 102, 2, 223, 223, 1006, 224, 404, 1001, 223, 1,
        223, 1108, 677, 677, 224, 1002, 223, 2, 223, 1005, 224, 419, 101, 1, 223, 223, 1008, 226,
        677, 224, 102, 2, 223, 223, 1006, 224, 434, 101, 1, 223, 223, 107, 226, 226, 224, 102, 2,
        223, 223, 1005, 224, 449, 1001, 223, 1, 223, 1007, 677, 677, 224, 1002, 223, 2, 223, 1006,
        224, 464, 1001, 223, 1, 223, 1007, 226, 226, 224, 1002, 223, 2, 223, 1005, 224, 479, 101,
        1, 223, 223, 1008, 226, 226, 224, 102, 2, 223, 223, 1006, 224, 494, 1001, 223, 1, 223, 8,
        226, 226, 224, 102, 2, 223, 223, 1006, 224, 509, 101, 1, 223, 223, 1107, 677, 677, 224,
        102, 2, 223, 223, 1005, 224, 524, 1001, 223, 1, 223, 1108, 226, 677, 224, 1002, 223, 2,
        223, 1006, 224, 539, 101, 1, 223, 223, 1107, 677, 226, 224, 1002, 223, 2, 223, 1006, 224,
        554, 101, 1, 223, 223, 1007, 677, 226, 224, 1002, 223, 2, 223, 1005, 224, 569, 101, 1, 223,
        223, 7, 677, 226, 224, 1002, 223, 2, 223, 1006, 224, 584, 101, 1, 223, 223, 107, 677, 677,
        224, 1002, 223, 2, 223, 1005, 224, 599, 1001, 223, 1, 223, 8, 226, 677, 224, 1002, 223, 2,
        223, 1005, 224, 614, 101, 1, 223, 223, 7, 677, 677, 224, 1002, 223, 2, 223, 1006, 224, 629,
        1001, 223, 1, 223, 1107, 226, 677, 224, 1002, 223, 2, 223, 1006, 224, 644, 101, 1, 223,
        223, 108, 226, 226, 224, 102, 2, 223, 223, 1005, 224, 659, 1001, 223, 1, 223, 8, 677, 226,
        224, 1002, 223, 2, 223, 1005, 224, 674, 101, 1, 223, 223, 4, 223, 99, 226,
    ];

    let mut output = Vec::new();
    run_program(
        &mut program.clone(),
        || 1,
        |a| {
            output.push(a);
        },
    );
    println!("Part 1: {}", output.pop().unwrap());

    run_program(
        &mut program.clone(),
        || 5,
        |a| {
            output.push(a);
        },
    );
    println!("Part 2: {}", output.pop().unwrap());
}
