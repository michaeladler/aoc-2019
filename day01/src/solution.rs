const INPUT: &'static str = include_str!("../input.txt");

#[inline]
fn calc_fuel(x: i64) -> i64 {
    (x / 3) - 2
}

pub fn part1() -> i64 {
    INPUT
        .lines()
        .map(|x| x.parse().unwrap())
        .map(|x| calc_fuel(x))
        .sum()
}

fn calc_fuel_rec(mass: i64) -> i64 {
    let mut total = 0;
    let mut start = mass;
    loop {
        let fuel = calc_fuel(start);
        if fuel <= 0 {
            return total;
        }
        start = fuel;
        total += fuel;
    }
}

pub fn part2() -> i64 {
    INPUT
        .lines()
        .map(|x| x.parse().unwrap())
        .map(|x| calc_fuel_rec(x))
        .sum()
}
