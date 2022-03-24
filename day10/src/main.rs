#[macro_use]
extern crate log;
extern crate env_logger;
extern crate num_rational;

use std::cmp::Ordering;
use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap, HashMap, HashSet};
use std::fs::File;
use std::io::Read;

use num_rational::Rational32;

#[derive(Debug, PartialEq, Hash, Eq)]
enum Quadrant {
    UpperRight,
    LowerRight,
    UpperLeft,
    LowerLeft,
}

type Point = (i32, i32);

#[derive(Eq, Debug)]
struct WeightedPoint {
    p: Point,
    weight: u32,
}

impl WeightedPoint {
    pub fn new(p: Point, weight: u32) -> Self {
        Self { p, weight }
    }
}

impl Ord for WeightedPoint {
    fn cmp(&self, other: &Self) -> Ordering {
        self.weight.cmp(&other.weight)
    }
}

impl PartialOrd for WeightedPoint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for WeightedPoint {
    fn eq(&self, other: &Self) -> bool {
        self.weight == other.weight
    }
}

fn parse_asteroids(s: &str) -> Vec<Point> {
    let mut result: Vec<Point> = Vec::new();

    let mut y = 0;
    for line in s.split('\n') {
        let mut x = 0;
        for c in line.chars() {
            if c == '#' {
                result.push((x, y));
            }
            x += 1;
        }
        y += 1;
    }
    return result;
}

fn quadrant(start: &Point, other: &Point) -> Quadrant {
    match (start.0.cmp(&other.0), start.1.cmp(&other.1)) {
        (Ordering::Equal, Ordering::Equal) => Quadrant::UpperRight,
        // (4,4) (4, 2) => directly on top, let's say UpperRight
        (Ordering::Equal, Ordering::Greater) => Quadrant::UpperRight,
        // (4,4) (5, 3) => upper right
        (Ordering::Less, Ordering::Greater) => Quadrant::UpperRight,
        // (4,4) (5, 4) => upper right
        (Ordering::Less, Ordering::Equal) => Quadrant::UpperRight,
        // (4,4) (5, 6) => lower right
        (Ordering::Less, Ordering::Less) => Quadrant::LowerRight,
        // (4,4) (4, 5) => directly below, let's say LowerRight
        (Ordering::Equal, Ordering::Less) => Quadrant::LowerRight,
        // (4,4) (3, 5) => lower left
        (Ordering::Greater, Ordering::Less) => Quadrant::LowerLeft,
        // (4,4) (2, 4) => directly left, let's say lower left
        (Ordering::Greater, Ordering::Equal) => Quadrant::LowerLeft,
        // (4,4) (2, 2) => upper left
        (Ordering::Greater, Ordering::Greater) => Quadrant::UpperLeft,
    }
}

fn count_visible_points(start: &Point, all: &Vec<Point>) -> usize {
    let mut slopes = HashMap::new();
    let mut vertical_points = HashSet::new();

    for other in all.iter() {
        if start == other {
            continue;
        }
        let quad = quadrant(start, &other);
        let dx = other.0 - start.0;
        if dx == 0 {
            vertical_points.insert(quad);
        } else {
            let dy = other.1 - start.1;
            let m = Rational32::new(dy, dx);
            debug!("start={:?}, other={:?}, m={}", start, other, m);

            let entry = slopes.entry(m).or_insert(HashSet::new());
            entry.insert(quad);
        }
    }
    debug!("slopes: {:?}, vertical_points: {:?}", slopes, vertical_points);

    let mut result = 0;
    for x in slopes.values() {
        result += x.len();
    }
    result += vertical_points.len();
    return result;
}

fn vaporize(start: &Point, all: &Vec<Point>) -> Vec<Point> {
    let mut top = BinaryHeap::new();
    let mut upper_right = BTreeMap::new();
    let mut lower_right = BTreeMap::new();
    let mut bottom = BinaryHeap::new();
    let mut lower_left = BTreeMap::new();
    let mut upper_left = BTreeMap::new();

    for other in all.iter() {
        if start == other {
            continue;
        }

        let dx = other.0 - start.0;
        let dy = other.1 - start.1;
        let quad = quadrant(start, &other);
        let dist = (dx * dx + dy * dy) as u32;
        let wp = WeightedPoint::new(*other, dist);

        if dx == 0 {
            if dy <= 0 {
                top.push(Reverse(wp));
            } else {
                bottom.push(Reverse(wp));
            }
        } else {
            let m = Rational32::new(dy, dx);
            debug!("start={:?}, other={:?}, m={}", start, other, m);
            match quad {
                Quadrant::UpperRight => {
                    upper_right.entry(m).or_insert(BinaryHeap::new()).push(Reverse(wp));
                }
                Quadrant::LowerRight => {
                    lower_right.entry(m).or_insert(BinaryHeap::new()).push(Reverse(wp));
                }
                Quadrant::LowerLeft => {
                    lower_left.entry(m).or_insert(BinaryHeap::new()).push(Reverse(wp));
                }
                Quadrant::UpperLeft => {
                    upper_left.entry(m).or_insert(BinaryHeap::new()).push(Reverse(wp));
                }
            };
        }
    }
    debug!("top: {:?}", top);
    debug!("upper_right: {:?}", upper_right);
    debug!("lower_right: {:?}", lower_right);
    debug!("bottom: {:?}", bottom);
    debug!("lower_left: {:?}", lower_left);
    debug!("upper_left: {:?}", upper_left);

    let mut vap_points = Vec::new();
    loop {
        let n = vap_points.len();

        if let Some(a) = top.pop() {
            vap_points.push(a.0.p);
        }
        for (_k, v) in upper_right.iter_mut() {
            if let Some(a) = v.pop() {
                vap_points.push(a.0.p);
            }
        }
        for (_k, v) in lower_right.iter_mut() {
            if let Some(a) = v.pop() {
                vap_points.push(a.0.p);
            }
        }
        if let Some(a) = bottom.pop() {
            vap_points.push(a.0.p);
        }
        for (_k, v) in lower_left.iter_mut() {
            if let Some(a) = v.pop() {
                vap_points.push(a.0.p);
            }
        }
        for (_k, v) in upper_left.iter_mut() {
            if let Some(a) = v.pop() {
                vap_points.push(a.0.p);
            }
        }

        if vap_points.len() <= n {
            break;
        }
    }
    return vap_points;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_parse_asteroids() {
        init();

        let s = &".#..#
.....
#####
....#
...##"[..];

        assert_eq!(
            parse_asteroids(s),
            vec![(1, 0), (4, 0), (0, 2), (1, 2), (2, 2), (3, 2), (4, 2), (4, 3), (3, 4), (4, 4)]
        );
    }

    #[test]
    fn test_count_visible_points() {
        init();

        let points = vec![(1, 0), (4, 0), (0, 2), (1, 2), (2, 2), (3, 2), (4, 2), (4, 3), (3, 4), (4, 4)];
        assert_eq!(count_visible_points(&points[0], &points), 7);
        assert_eq!(count_visible_points(&points[1], &points), 7);

        assert_eq!(count_visible_points(&points[2], &points), 6);
        assert_eq!(count_visible_points(&points[3], &points), 7);
        assert_eq!(count_visible_points(&points[4], &points), 7);
        assert_eq!(count_visible_points(&points[5], &points), 7);
        assert_eq!(count_visible_points(&points[6], &points), 5);

        assert_eq!(count_visible_points(&points[7], &points), 7);

        assert_eq!(count_visible_points(&points[8], &points), 8);
        assert_eq!(count_visible_points(&points[9], &points), 7);
    }

    #[test]
    fn test_vaporize() {
        init();

        let input = ".#..##.###...#######
##.############..##.
.#.######.########.#
.###.#######.####.#.
#####.##.#.##.###.##
..#####..#.#########
####################
#.####....###.#.#.##
##.#################
#####.##.###..####..
..######..##.#######
####.##.####...##..#
.#####..#.######.###
##...#.##########...
#.##########.#######
.####.#.###.###.#.##
....##.##.###..#####
.#.#.###########.###
#.#.#.#####.####.###
###.##.####.##.#..##";
        let points = parse_asteroids(&input[..]);
        let vap_points = vaporize(&(11, 13), &points);
        assert_eq!(vap_points[0], (11, 12));
        assert_eq!(vap_points[1], (12, 1));
        assert_eq!(vap_points[2], (12, 2));
        assert_eq!(vap_points[9], (12, 8));
        assert_eq!(vap_points[19], (16, 0));
        assert_eq!(vap_points[49], (16, 9));
        assert_eq!(vap_points[99], (10, 16));
        assert_eq!(vap_points[198], (9, 6));
        assert_eq!(vap_points[199], (8, 2));
        assert_eq!(vap_points[200], (10, 9));
        assert_eq!(vap_points[298], (11, 1));
    }
}

fn main() -> std::io::Result<()> {
    env_logger::init();

    let mut file = File::open("input.txt")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let asteroids = parse_asteroids(&contents);
    let mut cur_max = 0;
    let mut start = None;
    for p in asteroids.iter() {
        let count = count_visible_points(&p, &asteroids);
        if count > cur_max {
            cur_max = count;
            start = Some(p);
        }
    }
    println!("Part One: {}", cur_max);

    let vap_points = vaporize(&start.unwrap(), &asteroids);
    let poi = vap_points[199];

    println!("Part Two: {}", poi.0 * 100 + poi.1);

    Ok(())
}
