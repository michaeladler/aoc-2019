use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use log::{error, trace};

#[derive(Debug, PartialEq)]
pub enum IntcodeResult {
    /// Program needs more numbers to continue (number source "exhausted")
    SUSPENDED,
    /// Program terminated
    TERMINATED,
    /// EOF reached, but program did not send TERMINATE instruction
    EOF,
}

impl IntcodeResult {
    pub fn is_active(&self) -> bool {
        use IntcodeResult::*;
        match self {
            SUSPENDED => true,
            TERMINATED => false,
            EOF => false,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum ParameterMode {
    ABSOLUTE,
    IMMEDIATE,
    RELATIVE,
}

type Address = u64;

#[derive(Debug, PartialEq, Clone)]
pub struct IntcodeProgram {
    code: Vec<i64>,
    ip: usize,
    rel_base: i64,
    ram: HashMap<Address, i64>,
}

impl IntcodeProgram {
    pub fn new(code: Vec<i64>) -> IntcodeProgram {
        IntcodeProgram {
            code,
            ip: 0,
            rel_base: 0,
            ram: HashMap::new(),
        }
    }

    pub fn from_file(fname: &str) -> Option<IntcodeProgram> {
        let file = File::open(fname).unwrap();
        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();
        buf_reader.read_to_string(&mut contents).unwrap();
        let mut code = Vec::new();
        for s in contents.split(',') {
            if let Ok(n) = s.trim_end().parse::<i64>() {
                code.push(n);
            } else {
                error!("Failed to parse: {}", s);
                return None;
            }
        }
        code.shrink_to_fit();
        return Some(IntcodeProgram::new(code));
    }

    pub fn run(&mut self, input: &[i64], output: &mut Vec<i64>) -> IntcodeResult {
        let mut number_idx = 0;

        let n = self.code.len();
        while self.ip < n {
            let ip = self.ip;
            let (op_code, param_modes) = self.parse_instruction();
            trace!(
                "Processing op_code={}, param_modes={:?}",
                op_code,
                param_modes
            );
            match op_code {
                // add
                1 => {
                    let a = self.read_param(&param_modes, 0);
                    let b = self.read_param(&param_modes, 1);
                    let out_pos = self.read_out_pos(&param_modes, 2);
                    trace!("[Add] a={}, b={}, out_pos={}", a, b, out_pos);
                    trace!(
                        "Adding numbers {}, {} and storing result in {}",
                        a,
                        b,
                        out_pos
                    );
                    self.write_value(u64::try_from(out_pos).unwrap(), a + b);
                    self.ip += 4;
                }
                // multiply
                2 => {
                    let a = self.read_param(&param_modes, 0);
                    let b = self.read_param(&param_modes, 1);
                    let out_pos = self.read_out_pos(&param_modes, 2);
                    trace!("[Mul] a={}, b={}, out_pos={}", a, b, out_pos);
                    trace!(
                        "Multiplying numbers {}, {} and storing result in {}",
                        a,
                        b,
                        out_pos
                    );
                    self.write_value(u64::try_from(out_pos).unwrap(), a * b);
                    self.ip += 4;
                }
                // get number
                3 => {
                    let out_pos = self.read_out_pos(&param_modes, 0);
                    trace!(
                        "[Get] Getting number and storing it in position {}",
                        out_pos
                    );
                    if number_idx >= input.len() {
                        trace!("[Get] Need more numbers to continue");
                        return IntcodeResult::SUSPENDED;
                    }
                    let n = input[number_idx];
                    trace!("[Get] Received number at pos {}: {}", number_idx, n);
                    number_idx += 1;
                    self.write_value(u64::try_from(out_pos).unwrap(), n);
                    self.ip += 2;
                }
                // put number
                4 => {
                    let a = self.read_param(&param_modes, 0);
                    trace!("[Put] Appending {}", a);
                    output.push(a);
                    self.ip += 2;
                }
                // jump-if-true
                5 => {
                    let a = self.read_param(&param_modes, 0);
                    let b = self.read_param(&param_modes, 1);

                    trace!("[JUMP-IF-TRUE] a={}, b={}", a, b);
                    self.ip = match a {
                        0 => {
                            trace!("No jump");
                            self.ip + 3
                        }
                        _ => {
                            trace!("Jumping");
                            usize::try_from(b).expect("Number too large")
                        }
                    };
                }
                // jump-if-false
                6 => {
                    let a = self.read_param(&param_modes, 0);
                    let b = self.read_param(&param_modes, 1);

                    trace!("[JUMP-IF-FALSE] a={}, b={}", a, b);
                    self.ip = match a {
                        0 => {
                            trace!("Jumping");
                            usize::try_from(b).expect("Number too large")
                        }
                        _ => {
                            trace!("No jump");
                            self.ip + 3
                        }
                    };
                }
                // less-than
                7 => {
                    let a = self.read_param(&param_modes, 0);
                    let b = self.read_param(&param_modes, 1);
                    let out_pos = self.read_out_pos(&param_modes, 2);
                    trace!(
                        "[LT] Checking if {} < {} and storing result in {}",
                        a,
                        b,
                        out_pos
                    );
                    self.write_value(u64::try_from(out_pos).unwrap(), if a < b { 1 } else { 0 });
                    self.ip += 4;
                }
                // equals
                8 => {
                    let a = self.read_param(&param_modes, 0);
                    let b = self.read_param(&param_modes, 1);
                    let out_pos = self.read_out_pos(&param_modes, 2);
                    trace!(
                        "[EQ] Checking if {} == {} and storing result in {}",
                        a,
                        b,
                        out_pos
                    );
                    self.write_value(u64::try_from(out_pos).unwrap(), if a == b { 1 } else { 0 });
                    self.ip += 4;
                }
                // adjust relative base
                9 => {
                    let a = self.read_param(&param_modes, 0);
                    let new_rel_base = self.rel_base + a;
                    trace!("[BASE] Adjusting base: {} -> {}", self.rel_base, new_rel_base);
                    self.rel_base = new_rel_base;
                    self.ip += 2;
                }
                99 => {
                    trace!("HALT instruction");
                    return IntcodeResult::TERMINATED;
                }
                _ => panic!("Unsupported op code: {}", op_code),
            }
            trace!("ip: {} -> {}, base: {}", ip, self.ip, self.rel_base);
        }
        return IntcodeResult::EOF;
    }

    fn parse_instruction(&self) -> (i32, Vec<ParameterMode>) {
        let mut instruction = i32::try_from(self.code[self.ip]).expect("Instruction too large");
        trace!("Parsing instruction from value {}", instruction);
        let op_code = instruction % 100;

        instruction -= op_code;
        instruction = instruction / 100;

        let mut param_modes = Vec::new();
        while instruction > 0 {
            let mode = instruction % 10;
            param_modes.push(match mode {
                0 => ParameterMode::ABSOLUTE,
                1 => ParameterMode::IMMEDIATE,
                2 => ParameterMode::RELATIVE,
                _ => panic!("Unsupported parameter mode"),
            });
            instruction -= mode;
            instruction = instruction / 10;
        }
        return (op_code, param_modes);
    }

    fn read_param(&self, param_modes: &Vec<ParameterMode>, i: usize) -> i64 {
        let val = self.code[self.ip + 1 + i];
        match param_modes.get(i) {
            // position mode
            Some(ParameterMode::ABSOLUTE) | None => self.read_value(u64::try_from(val).unwrap()),
            // immediate mode
            Some(ParameterMode::IMMEDIATE) => val,
            // relative mode
            Some(ParameterMode::RELATIVE) => {
                self.read_value(u64::try_from(self.rel_base + val).unwrap())
            }
        }
    }

    fn read_out_pos(&self, param_modes: &Vec<ParameterMode>, i: usize) -> i64 {
        let mode = param_modes.get(i);
        trace!("Reading out pos using mode {:?}", mode);
        let val = self.code[self.ip + 1 + i];
        match mode {
            Some(ParameterMode::RELATIVE) => self.rel_base + val,
            _ => val,
        }
    }

    fn read_value(&self, address: Address) -> i64 {
        let n = self.code.len();
        if let Ok(small_address) = usize::try_from(address) {
            if small_address < n {
                let val = self.code[small_address];
                trace!("Reading from code: address={}, val={}", address, val);
                return val;
            }
        }
        let val = *self.ram.get(&address).unwrap_or(&0);
        trace!("Reading from ram: address={}, val={}", address, val);
        return val;
    }

    fn write_value(&mut self, address: Address, val: i64) {
        let n = self.code.len();
        if let Ok(small_address) = usize::try_from(address) {
            if small_address < n {
                trace!("Writing memory to code: address={}, val={}", address, val);
                self.code[small_address] = val;
                return;
            }
        }
        trace!("Writing memory to ram: address={}, val={}", address, val);
        self.ram.insert(address, val);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_run() {
        init();
        let mut result = Vec::new();

        let code = vec![1, 0, 0, 0, 99];
        let mut program = IntcodeProgram::new(code.clone());
        program.run(&mut vec![42], &mut result);
        assert_eq!(program.code, [2, 0, 0, 0, 99]);

        let code = vec![2, 3, 0, 3, 99];
        let mut program = IntcodeProgram::new(code.clone());
        program.run(&mut vec![42], &mut result);
        assert_eq!(program.code, [2, 3, 0, 6, 99]);

        let code = vec![2, 4, 4, 5, 99, 0];
        let mut program = IntcodeProgram::new(code.clone());
        program.run(&mut vec![42], &mut result);
        assert_eq!(program.code, [2, 4, 4, 5, 99, 9801]);

        let code = vec![1, 1, 1, 4, 99, 5, 6, 0, 99];
        let mut program = IntcodeProgram::new(code.clone());
        program.run(&mut vec![42], &mut result);
        assert_eq!(program.code, [30, 1, 1, 4, 2, 5, 6, 0, 99]);
    }

    #[test]
    fn test_run_program_parameter_mode() {
        init();
        let mut result = Vec::new();

        let code = vec![1002, 4, 3, 4, 33];
        let mut program = IntcodeProgram::new(code.clone());
        program.run(&mut vec![42], &mut result);
        assert_eq!(program.code, [1002, 4, 3, 4, 99]);
    }

    #[test]
    fn test_jump_if_equal_position_mode() {
        init();
        let mut _output: Vec<i64> = Vec::new();

        // Using position mode, consider whether the input is equal to 8; output 1 (if it is) or 0 (if it is not).
        let code = vec![3, 9, 8, 9, 10, 9, 4, 9, 99, -1, 8];
        let mut program = IntcodeProgram::new(code.clone());

        let mut output = Vec::new();
        program.run(&mut vec![8], &mut output);
        assert_eq!(output, vec![1]);

        output.clear();
        let mut program = IntcodeProgram::new(code.clone());
        program.run(&mut vec![7], &mut output);
        assert_eq!(output, vec![0]);
    }

    #[test]
    fn test_jump_if_less_than_position_mode() {
        init();
        let mut _output: Vec<i64> = Vec::new();

        // Using position mode, consider whether the input is less than 8; output 1 (if it is) or 0 (if it is not).
        let code = vec![3, 9, 7, 9, 10, 9, 4, 9, 99, -1, 8];
        let mut program = IntcodeProgram::new(code.clone());

        let mut output = Vec::new();
        program.run(&mut vec![7], &mut output);
        assert_eq!(output, vec![1]);

        output.clear();
        let mut program = IntcodeProgram::new(code.clone());
        program.run(&mut vec![8], &mut output);
        assert_eq!(output, vec![0]);
    }

    #[test]
    fn test_jump_if_equal_immediate_mode() {
        init();
        let mut _output: Vec<i64> = Vec::new();

        // Using immediate mode, consider whether the input is equal to 8; output 1 (if it is) or 0 (if it is not)
        let code = vec![3, 3, 1108, -1, 8, 3, 4, 3, 99];
        let mut program = IntcodeProgram::new(code.clone());

        let mut output = Vec::new();
        program.run(&mut vec![8], &mut output);
        assert_eq!(output, vec![1]);

        output.clear();
        let mut program = IntcodeProgram::new(code.clone());
        program.run(&mut vec![7], &mut output);
        assert_eq!(output, vec![0]);
    }

    #[test]
    fn test_jump_if_less_than_immediate_mode() {
        init();
        let mut _output: Vec<i64> = Vec::new();

        //  Using immediate mode, consider whether the input is less than 8; output 1 (if it is) or 0 (if it is not).
        let code = vec![3, 3, 1107, -1, 8, 3, 4, 3, 99];
        let mut program = IntcodeProgram::new(code.clone());

        let mut output = Vec::new();
        program.run(&mut vec![7], &mut output);
        assert_eq!(output, vec![1]);

        output.clear();
        let mut program = IntcodeProgram::new(code.clone());
        program.run(&mut vec![8], &mut output);
        assert_eq!(output, vec![0]);
    }

    #[test]
    fn test_jumps_position_mode() {
        init();
        let mut _output: Vec<i64> = Vec::new();

        // Here are some jump tests that take an input, then output 0 if the input was zero or 1 if the input was non-zero:
        let code = vec![3, 12, 6, 12, 15, 1, 13, 14, 13, 4, 13, 99, -1, 0, 1, 9];
        let mut program = IntcodeProgram::new(code.clone());

        let mut output = Vec::new();
        program.run(&mut vec![0], &mut output);
        assert_eq!(output, vec![0]);

        output.clear();
        let mut program = IntcodeProgram::new(code.clone());
        program.run(&mut vec![2], &mut output);
        assert_eq!(output, vec![1]);
    }

    #[test]
    fn test_jumps_immediate_mode() {
        init();
        let mut _output: Vec<i64> = Vec::new();

        // Here are some jump tests that take an input, then output 0 if the input was zero or 1 if the input was non-zero:
        let code = vec![3, 3, 1105, -1, 9, 1101, 0, 0, 12, 4, 12, 99, 1];
        let mut program = IntcodeProgram::new(code.clone());
        let mut output = Vec::new();
        program.run(&mut vec![0], &mut output);
        assert_eq!(output, vec![0]);

        output.clear();
        let mut program = IntcodeProgram::new(code.clone());
        program.run(&mut vec![2], &mut output);
        assert_eq!(output, vec![1]);
    }

    #[test]
    fn test_quine() {
        let code = vec![
            109, 1, 204, -1, 1001, 100, 1, 100, 1008, 100, 16, 101, 1006, 101, 0, 99,
        ];
        let mut program = IntcodeProgram::new(code.clone());
        let mut output = Vec::new();
        program.run(&mut vec![], &mut output);
        assert_eq!(output, code);
    }

    #[test]
    fn test_opcode9() {
        let code = vec![1102, 34915192, 34915192, 7, 4, 7, 99, 0];
        let mut program = IntcodeProgram::new(code.clone());
        let mut output = Vec::new();
        program.run(&mut vec![], &mut output);
        assert_eq!(output, vec![1219070632396864]);
    }

    #[test]
    fn test_extra_memory() {
        let code = vec![104, 1125899906842624, 99];
        let mut program = IntcodeProgram::new(code.clone());
        let mut output = Vec::new();
        program.run(&mut vec![], &mut output);
        assert_eq!(output, vec![1125899906842624]);
    }

    #[test]
    fn test_boost() {
        init();

        // this tests *every* feature
        let code = vec![
            1102, 34463338, 34463338, 63, 1007, 63, 34463338, 63, 1005, 63, 53, 1101, 0, 3, 1000,
            109, 988, 209, 12, 9, 1000, 209, 6, 209, 3, 203, 0, 1008, 1000, 1, 63, 1005, 63, 65,
            1008, 1000, 2, 63, 1005, 63, 904, 1008, 1000, 0, 63, 1005, 63, 58, 4, 25, 104, 0, 99,
            4, 0, 104, 0, 99, 4, 17, 104, 0, 99, 0, 0, 1101, 0, 36, 1015, 1102, 1, 387, 1028, 1101,
            24, 0, 1016, 1101, 0, 23, 1008, 1102, 1, 35, 1012, 1102, 1, 554, 1023, 1101, 29, 0,
            1003, 1101, 27, 0, 1011, 1101, 25, 0, 1000, 1101, 0, 38, 1018, 1102, 20, 1, 1019, 1102,
            28, 1, 1005, 1102, 1, 619, 1026, 1102, 1, 22, 1004, 1101, 0, 0, 1020, 1101, 0, 31,
            1009, 1102, 1, 783, 1024, 1102, 1, 33, 1001, 1102, 616, 1, 1027, 1102, 1, 21, 1006,
            1101, 32, 0, 1013, 1102, 39, 1, 1014, 1102, 1, 378, 1029, 1101, 774, 0, 1025, 1102, 1,
            1, 1021, 1102, 30, 1, 1007, 1102, 37, 1, 1002, 1102, 1, 26, 1017, 1101, 0, 557, 1022,
            1102, 1, 34, 1010, 109, 13, 2101, 0, -5, 63, 1008, 63, 23, 63, 1005, 63, 203, 4, 187,
            1105, 1, 207, 1001, 64, 1, 64, 1002, 64, 2, 64, 109, -14, 2107, 28, 4, 63, 1005, 63,
            225, 4, 213, 1106, 0, 229, 1001, 64, 1, 64, 1002, 64, 2, 64, 109, 10, 1207, -3, 20, 63,
            1005, 63, 245, 1106, 0, 251, 4, 235, 1001, 64, 1, 64, 1002, 64, 2, 64, 109, 8, 1205, 3,
            263, 1105, 1, 269, 4, 257, 1001, 64, 1, 64, 1002, 64, 2, 64, 109, -9, 1207, -7, 34, 63,
            1005, 63, 287, 4, 275, 1105, 1, 291, 1001, 64, 1, 64, 1002, 64, 2, 64, 109, -4, 2102,
            1, -3, 63, 1008, 63, 32, 63, 1005, 63, 311, 1105, 1, 317, 4, 297, 1001, 64, 1, 64,
            1002, 64, 2, 64, 109, 21, 21101, 40, 0, -6, 1008, 1019, 43, 63, 1005, 63, 337, 1106, 0,
            343, 4, 323, 1001, 64, 1, 64, 1002, 64, 2, 64, 109, -26, 1202, 7, 1, 63, 1008, 63, 21,
            63, 1005, 63, 365, 4, 349, 1106, 0, 369, 1001, 64, 1, 64, 1002, 64, 2, 64, 109, 26,
            2106, 0, 3, 4, 375, 1001, 64, 1, 64, 1105, 1, 387, 1002, 64, 2, 64, 109, -9, 21108, 41,
            40, 3, 1005, 1019, 407, 1001, 64, 1, 64, 1106, 0, 409, 4, 393, 1002, 64, 2, 64, 109,
            13, 1205, -8, 423, 4, 415, 1106, 0, 427, 1001, 64, 1, 64, 1002, 64, 2, 64, 109, -19,
            21107, 42, 41, 5, 1005, 1015, 447, 1001, 64, 1, 64, 1106, 0, 449, 4, 433, 1002, 64, 2,
            64, 109, -3, 2102, 1, -5, 63, 1008, 63, 37, 63, 1005, 63, 471, 4, 455, 1105, 1, 475,
            1001, 64, 1, 64, 1002, 64, 2, 64, 109, -2, 1201, 0, 0, 63, 1008, 63, 28, 63, 1005, 63,
            497, 4, 481, 1105, 1, 501, 1001, 64, 1, 64, 1002, 64, 2, 64, 109, 8, 2107, 29, -8, 63,
            1005, 63, 521, 1001, 64, 1, 64, 1106, 0, 523, 4, 507, 1002, 64, 2, 64, 109, -3, 1208,
            -3, 30, 63, 1005, 63, 541, 4, 529, 1106, 0, 545, 1001, 64, 1, 64, 1002, 64, 2, 64, 109,
            4, 2105, 1, 9, 1105, 1, 563, 4, 551, 1001, 64, 1, 64, 1002, 64, 2, 64, 109, 9, 1206,
            -3, 581, 4, 569, 1001, 64, 1, 64, 1106, 0, 581, 1002, 64, 2, 64, 109, -8, 1201, -9, 0,
            63, 1008, 63, 23, 63, 1005, 63, 605, 1001, 64, 1, 64, 1106, 0, 607, 4, 587, 1002, 64,
            2, 64, 109, 21, 2106, 0, -9, 1106, 0, 625, 4, 613, 1001, 64, 1, 64, 1002, 64, 2, 64,
            109, -35, 2108, 31, 8, 63, 1005, 63, 647, 4, 631, 1001, 64, 1, 64, 1105, 1, 647, 1002,
            64, 2, 64, 109, 2, 1202, 0, 1, 63, 1008, 63, 30, 63, 1005, 63, 667, 1105, 1, 673, 4,
            653, 1001, 64, 1, 64, 1002, 64, 2, 64, 109, 17, 21108, 43, 43, -4, 1005, 1016, 691, 4,
            679, 1106, 0, 695, 1001, 64, 1, 64, 1002, 64, 2, 64, 109, -14, 1208, -1, 30, 63, 1005,
            63, 711, 1106, 0, 717, 4, 701, 1001, 64, 1, 64, 1002, 64, 2, 64, 109, 6, 21101, 44, 0,
            -1, 1008, 1011, 44, 63, 1005, 63, 739, 4, 723, 1105, 1, 743, 1001, 64, 1, 64, 1002, 64,
            2, 64, 109, -15, 2108, 30, 8, 63, 1005, 63, 759, 1106, 0, 765, 4, 749, 1001, 64, 1, 64,
            1002, 64, 2, 64, 109, 27, 2105, 1, 0, 4, 771, 1001, 64, 1, 64, 1105, 1, 783, 1002, 64,
            2, 64, 109, -9, 1206, 6, 795, 1105, 1, 801, 4, 789, 1001, 64, 1, 64, 1002, 64, 2, 64,
            109, 4, 21102, 45, 1, -7, 1008, 1012, 45, 63, 1005, 63, 823, 4, 807, 1105, 1, 827,
            1001, 64, 1, 64, 1002, 64, 2, 64, 109, -14, 21102, 46, 1, 5, 1008, 1010, 43, 63, 1005,
            63, 851, 1001, 64, 1, 64, 1105, 1, 853, 4, 833, 1002, 64, 2, 64, 109, -1, 2101, 0, 1,
            63, 1008, 63, 25, 63, 1005, 63, 873, 1105, 1, 879, 4, 859, 1001, 64, 1, 64, 1002, 64,
            2, 64, 109, 9, 21107, 47, 48, -3, 1005, 1010, 897, 4, 885, 1105, 1, 901, 1001, 64, 1,
            64, 4, 64, 99, 21101, 0, 27, 1, 21101, 915, 0, 0, 1106, 0, 922, 21201, 1, 57526, 1,
            204, 1, 99, 109, 3, 1207, -2, 3, 63, 1005, 63, 964, 21201, -2, -1, 1, 21101, 942, 0, 0,
            1106, 0, 922, 21201, 1, 0, -1, 21201, -2, -3, 1, 21101, 957, 0, 0, 1106, 0, 922, 22201,
            1, -1, -2, 1105, 1, 968, 21202, -2, 1, -2, 109, -3, 2106, 0, 0,
        ];
        let mut program = IntcodeProgram::new(code);
        let mut output = Vec::new();
        program.run(&mut vec![1], &mut output);
        assert_eq!(output, vec![3380552333]);
    }

    #[test]
    fn test_jumps_large_1() {
        init();

        // This program uses an input instruction to ask for a single number. The program will then output 999 if the input value is below 8, output 1000 if the input value is equal to 8, or output 1001 if the input value is greater than 8.
        let mut program = IntcodeProgram::new(vec![
            3, 21, 1008, 21, 8, 20, 1005, 20, 22, 107, 8, 21, 20, 1006, 20, 31, 1106, 0, 36, 98, 0,
            0, 1002, 21, 125, 20, 4, 20, 1105, 1, 46, 104, 999, 1105, 1, 46, 1101, 1000, 1, 20, 4,
            20, 1105, 1, 46, 98, 99,
        ]);

        let mut output = Vec::new();
        program.run(&mut vec![7], &mut output);
        assert_eq!(output, vec![999]);
    }

    #[test]
    fn test_jumps_large_2() {
        init();

        // This program uses an input instruction to ask for a single number. The program will then output 999 if the input value is below 8, output 1000 if the input value is equal to 8, or output 1001 if the input value is greater than 8.
        let mut program = IntcodeProgram::new(vec![
            3, 21, 1008, 21, 8, 20, 1005, 20, 22, 107, 8, 21, 20, 1006, 20, 31, 1106, 0, 36, 98, 0,
            0, 1002, 21, 125, 20, 4, 20, 1105, 1, 46, 104, 999, 1105, 1, 46, 1101, 1000, 1, 20, 4,
            20, 1105, 1, 46, 98, 99,
        ]);

        let mut output = Vec::new();
        program.run(&mut vec![8], &mut output);
        assert_eq!(output, vec![1000]);
    }

    #[test]
    fn test_jumps_large_3() {
        init();

        // This program uses an input instruction to ask for a single number. The program will then output 999 if the input value is below 8, output 1000 if the input value is equal to 8, or output 1001 if the input value is greater than 8.
        let mut program = IntcodeProgram::new(vec![
            3, 21, 1008, 21, 8, 20, 1005, 20, 22, 107, 8, 21, 20, 1006, 20, 31, 1106, 0, 36, 98, 0,
            0, 1002, 21, 125, 20, 4, 20, 1105, 1, 46, 104, 999, 1105, 1, 46, 1101, 1000, 1, 20, 4,
            20, 1105, 1, 46, 98, 99,
        ]);

        let mut output = Vec::new();
        program.run(&mut vec![9], &mut output);
        assert_eq!(output, vec![1001]);
    }
}
