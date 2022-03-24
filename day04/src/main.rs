fn is_six_digits(n: usize) -> bool {
    return n >= 100000 && n <= 999999;
}

#[test]
fn test_is_six_digits() {
    assert_eq!(is_six_digits(111111), true);
    assert_eq!(is_six_digits(223450), true);
    assert_eq!(is_six_digits(123789), true);
    assert_eq!(is_six_digits(99999), false);
    assert_eq!(is_six_digits(999999 + 1), false);
}

fn has_two_adjacent_digits(digits: &[usize]) -> bool {
    let n = digits.len();
    for i in 0..n - 1 {
        if digits[i] == digits[i + 1] {
            return true;
        }
    }
    return false;
}

#[test]
fn test_has_two_adjacent_digits() {
    assert_eq!(has_two_adjacent_digits(&[1, 1, 1, 1, 1, 1]), true);
    assert_eq!(has_two_adjacent_digits(&[2, 2, 3, 4, 5, 0]), true);
    assert_eq!(has_two_adjacent_digits(&[1, 2, 3, 7, 8, 9]), false);
}

fn has_two_adjacent_digits_strict(digits: &[usize]) -> bool {
    let n = digits.len();
    for i in 0..n - 1 {
        if digits[i] == digits[i + 1] {
            if i >= 1 {
                if digits[i - 1] == digits[i] {
                    continue;
                }
            }
            if i + 2 < n {
                if digits[i + 1] == digits[i + 2] {
                    continue;
                }
            }
            return true;
        }
    }
    return false;
}

#[test]
fn test_has_two_adjacent_digits_strict() {
    assert_eq!(has_two_adjacent_digits_strict(&[1, 1, 2, 2, 3, 3]), true);
    assert_eq!(has_two_adjacent_digits_strict(&[1, 2, 3, 4, 4, 4]), false);
    assert_eq!(has_two_adjacent_digits_strict(&[1, 1, 1, 1, 2, 2]), true);
}

fn main() {
    let lower = 147981;
    let upper = 691423;

    let mut count1 = 0;
    let mut count2 = 0;
    for i in 0..10 {
        for j in i..10 {
            for k in j..10 {
                for l in k..10 {
                    for m in l..10 {
                        for n in m..10 {
                            let candidate = n + 10 * m + 100 * l + 1000 * k + 10000 * j + 100000 * i;

                            /*
                            It is a six-digit number.
                            The value is within the range given in your puzzle input.
                            Two adjacent digits are the same (like 22 in 122345).
                            Going from left to right, the digits never decrease; they only ever increase or stay the same (like 111123 or 135679).
                             */
                            if candidate >= lower && candidate <= upper && is_six_digits(candidate) {
                                let digits = [i, j, k, l, m, n];
                                if has_two_adjacent_digits(&digits) {
                                    count1 += 1;
                                }
                                if has_two_adjacent_digits_strict(&digits) {
                                    count2 += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    println!("Solution Part 1: {}", count1);
    println!("Solution Part 2: {}", count2);
}
