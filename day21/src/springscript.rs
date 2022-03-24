const NEWLINE: i64 = 10;
const WHITESPACE: i64 = (' ' as u8) as i64;

#[derive(Debug)]
/// Read and write
pub enum RW {
    T,
    J,
}

impl RW {
    pub fn encode(&self) -> u8 {
        use RW::*;
        match self {
            T => 'T' as u8,
            J => 'J' as u8,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
/// Read only
pub enum RR {
    A,
    B,
    C,
    D,
    // part 2
    E,
    F,
    G,
    H,
    I,
    // also writeable
    T,
    J,
}

impl RR {
    pub fn encode(&self) -> u8 {
        use RR::*;
        match self {
            A => 'A' as u8,
            B => 'B' as u8,
            C => 'C' as u8,
            D => 'D' as u8,
            E => 'E' as u8,
            F => 'F' as u8,
            G => 'G' as u8,
            H => 'H' as u8,
            I => 'I' as u8,
            T => 'T' as u8,
            J => 'J' as u8,
        }
    }
}

#[derive(Debug)]
// There are 6*2*3 = 36 different instructions
pub enum Instruction {
    AND(RR, RW),
    OR(RR, RW),
    NOT(RR, RW),
}

impl Instruction {
    pub fn encode(&self) -> Vec<i64> {
        use Instruction::*;
        match self {
            AND(x, y) => vec![
                ('A' as u8) as i64,
                ('N' as u8) as i64,
                ('D' as u8) as i64,
                WHITESPACE,
                x.encode() as i64,
                WHITESPACE,
                y.encode() as i64,
            ],
            OR(x, y) => vec![
                ('O' as u8) as i64,
                ('R' as u8) as i64,
                WHITESPACE,
                x.encode() as i64,
                WHITESPACE,
                y.encode() as i64,
            ],
            NOT(x, y) => vec![
                ('N' as u8) as i64,
                ('O' as u8) as i64,
                ('T' as u8) as i64,
                WHITESPACE,
                x.encode() as i64,
                WHITESPACE,
                y.encode() as i64,
            ],
        }
    }
}

#[derive(Debug)]
pub struct Springscript {
    pub instructions: Vec<Instruction>,
}

impl Springscript {
    pub fn new(instructions: Vec<Instruction>) -> Self {
        Self { instructions }
    }

    // encode to Intcode input
    pub fn encode(&self) -> Vec<i64> {
        let mut result: Vec<i64> = Vec::new();
        for instruction in &self.instructions {
            let line: Vec<i64> = instruction.encode();
            for x in &line {
                result.push(*x);
            }
            result.push(NEWLINE);
        }
        return result;
    }
}
