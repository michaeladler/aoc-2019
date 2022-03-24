use bitset_core::BitSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
/// SmallAsciiBitset can store letters and digits
pub struct SmallAsciiBitset {
    data: u64,
}

impl SmallAsciiBitset {
    pub fn new() -> Self {
        Self { data: 0 }
    }

    pub fn from(c: char) -> Self {
        let mut result = Self { data: 0 };
        result.insert(c);
        return result;
    }

    fn translate(c: char) -> usize {
        // z is 122, a is 96, Z is 90, A is 65, '9' is 57, '0' is 48
        let d = c as u8;
        debug_assert!((d >= 96 && d <= 122) || (d >= 65 && d <= 90) || (d >= 48 && d <= 57));
        match d {
            48..=57 => (d - 48) as usize,       // digits map to 0-9
            65..=90 => (d - 65 + 10) as usize,  // uppercase maps to 10-35
            96..=122 => (d - 96 + 36) as usize, // lowercase maps to 36-62
            _ => panic!("Invalid character"),
        }
    }

    pub fn insert(&mut self, c: char) {
        self.data.bit_set(SmallAsciiBitset::translate(c));
    }

    pub fn contains(&self, c: char) -> bool {
        self.data.bit_test(SmallAsciiBitset::translate(c))
    }

    pub fn len(&self) -> usize {
        self.data.bit_count()
    }

    pub fn union(&mut self, other: &Self) {
        self.data.bit_or(&other.data);
    }
}
