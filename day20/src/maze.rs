use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;

use log::{debug, info, trace};

const WALL: char = '#';
const PASSAGE: char = '.';

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Portal {
    pub first: char,
    pub second: char,
}

impl Portal {
    pub fn new(first: char, second: char) -> Self {
        Self { first, second }
    }

    pub fn reverse(&self) -> Self {
        Self {
            first: self.second,
            second: self.first,
        }
    }
}

impl fmt::Display for Portal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.first, self.second)
    }
}

pub struct Grid {
    // first dimension is row, second dimension is col
    inner: Vec<Vec<char>>,
    _col_count: usize,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Point {
    pub row: usize,
    pub col: usize,
}

impl Grid {
    pub fn row_count(&self) -> usize {
        self.inner.len()
    }

    pub fn col_count(&self) -> usize {
        self._col_count
    }

    pub fn from_string(s: &str) -> Self {
        let mut inner: Vec<Vec<char>> = Vec::new();
        let mut max_col_count = 0;
        for line in s.lines() {
            let mut row = Vec::new();
            for c in line.trim_end().chars() {
                row.push(c);
            }
            let n = row.len();
            if n > max_col_count {
                max_col_count = n;
            }
            inner.push(row);
        }
        Self {
            inner,
            _col_count: max_col_count,
        }
    }

    /// Returns the entrance point for the given portal.
    /// The returned point is chosen such that it is located on a PASSAGE.
    pub fn find_portals(&self, portal: &Portal) -> Vec<Point> {
        let mut portals = Vec::with_capacity(2);
        // labels can oriented horizontally (left-to-right) or vertically (top-to-bottom)
        for (row_num, row) in self.inner.iter().enumerate() {
            for (col, &c) in row.iter().enumerate() {
                if c == portal.first {
                    if let Some(true) = row.get(col + 1).map(move |val| *val == portal.second) {
                        // horizontal
                        let mut found = false;
                        if let Some(&c) = self.inner[row_num].get(col + 2) {
                            if c == PASSAGE {
                                // passage is east
                                portals.push(Point {
                                    row: row_num,
                                    col: col + 2,
                                });
                                found = true;
                            }
                        }
                        if !found {
                            // passage is west
                            portals.push(Point {
                                row: row_num,
                                col: col - 1,
                            });
                        }
                    } else if let Some(next_row) = self.inner.get(row_num + 1) {
                        if let Some(true) = next_row.get(col).map(move |val| *val == portal.second)
                        {
                            // vertical
                            let mut found = false;
                            if let Some(next_next_row) = self.inner.get(row_num + 2) {
                                if let Some(&c) = next_next_row.get(col) {
                                    if c == PASSAGE {
                                        // passage is south
                                        portals.push(Point {
                                            row: row_num + 2,
                                            col,
                                        });
                                        found = true;
                                    }
                                }
                            }
                            if !found {
                                // passage is north
                                portals.push(Point {
                                    row: row_num - 1,
                                    col,
                                });
                            }
                        }
                    }
                }
            }
        }
        return portals;
    }

    /// Determine whether `portal` is outer or inner.
    fn is_outer_portal(&self, point: &Point) -> bool {
        let row_idx = point.row;
        let col_idx = point.col;
        // first point of portal is on PASSAGE, hence all numbers are +1
        let is_outer = row_idx <= 2 // first two rows are 'outer'
            || row_idx + 3 >= self.row_count() // last two rows are 'outer'
            || col_idx <= 2 // first two cols are 'outer'
            || col_idx + 3 >= self.col_count(); // last two rows are 'outer'
        debug!("{} is_outer {}", point, is_outer);
        return is_outer;
    }

    fn neighbors(&self, current: Point) -> Vec<Point> {
        let mut result: Vec<Point> = Vec::new();
        let nums: [(i64, i64); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
        for (drow, dcol) in &nums {
            let row_idx = (current.row as i64) + *drow;
            if row_idx < 0 {
                continue;
            }
            let row_idx = row_idx as usize;
            if let Some(col) = self.inner.get(row_idx) {
                let col_idx = (current.col as i64) + *dcol;
                if col_idx < 0 {
                    continue;
                }
                let col_idx = col_idx as usize;
                if let Some(&c) = col.get(col_idx) {
                    if c != WALL {
                        result.push(Point {
                            row: row_idx,
                            col: col_idx,
                        });
                    }
                }
            }
        }
        return result;
    }

    // `point` is the first char of a portal. returns the two letters of the portal,
    // i.e. it discovers the second char.
    fn discover_portal(&self, point: Point) -> Option<Portal> {
        trace!("discovering portal at {:?}", point);
        let c = self.inner[point.row][point.col];
        if !c.is_ascii() || !c.is_uppercase() {
            return None;
        }
        for np in self.neighbors(point) {
            let d = self.inner[np.row][np.col];
            if d.is_ascii() && d.is_uppercase() {
                return Some(Portal::new(c, d));
            }
        }
        return None;
    }

    /// Find all portals reachable from `start` and return their distance.
    pub fn bfs(&self, start: Point) -> HashMap<Point, Point> {
        // bfs
        let mut queue: VecDeque<(Point, usize)> = VecDeque::new();
        let mut discovered: HashSet<Point> = HashSet::new();

        // value: parent Point and distance
        let mut parents: HashMap<Point, Point> = HashMap::new();
        queue.push_back((start, 0));
        discovered.insert(start);

        while let Some((current, dist)) = queue.pop_front() {
            debug!("visiting {:?}, dist {}", current, dist);
            for point in self.neighbors(current) {
                if !discovered.contains(&point) {
                    discovered.insert(point);
                    parents.insert(point, current);

                    let c = self.inner[point.row][point.col];
                    debug!("discovered {:?}: {}", point, c);

                    if c == PASSAGE {
                        queue.push_back((point, dist + 1));
                        continue;
                    }
                    // portal, but not the portal where we started from
                    // add the point **before** it to the result
                    // now go through portal
                    let portal = self.discover_portal(point).unwrap();
                    debug!("found portal {}", portal);

                    let mut portal_points = self.find_portals(&portal);
                    // portal string might be reversed
                    if portal_points.is_empty() {
                        portal_points = self.find_portals(&portal.reverse());
                    }
                    if let Some(other_portal_point) = portal_points.iter().find(|&&p| p != current)
                    {
                        // go through portal
                        info!(
                            "portal {} at {:?} is connected with point {:?}. adding to queue.",
                            portal, current, other_portal_point
                        );
                        if !discovered.contains(other_portal_point) {
                            discovered.insert(*other_portal_point);
                            parents.insert(*other_portal_point, current);
                            queue.push_back((*other_portal_point, dist + 1));
                        }
                    }
                }
            }
        }
        return parents;
    }

    pub fn recursive_walk(&self, start: Point) -> usize {
        // bfs
        let mut queue: VecDeque<IntermediateResult> = VecDeque::new();
        let mut discovered: HashSet<Discovered> = HashSet::new();

        queue.push_back(IntermediateResult {
            current: start,
            distance: 0,
            level: 0,
        });
        discovered.insert(Discovered {
            point: start,
            level: 0,
        });

        let mut best_dist = usize::MAX;
        while let Some(item) = queue.pop_front() {
            debug!("popped from queue: {}", item);
            if item.distance > best_dist {
                trace!("cutting off branch");
                continue;
            }

            for point in self.neighbors(item.current) {
                let my_discover = Discovered {
                    point,
                    level: item.level,
                };
                if discovered.contains(&my_discover) {
                    trace!("{:?} seen before", my_discover);
                    continue;
                }

                let c = self.inner[point.row][point.col];
                if c == PASSAGE {
                    trace!("neighbor is a new passage");
                    queue.push_back(IntermediateResult {
                        current: my_discover.point,
                        level: my_discover.level,
                        distance: item.distance + 1,
                    });
                    discovered.insert(my_discover);
                    continue;
                }
                if item.distance <= 1 {
                    // we just started, so this is AA
                    continue;
                }

                let portal = self.discover_portal(point).unwrap();
                if portal == Portal::new('Z', 'Z') && item.level == 0 {
                    let d = item.distance;
                    debug!("reached ZZ after {} steps", d);
                    if item.distance < best_dist {
                        info!("new best distance: {}", d);
                        best_dist = d;
                    }
                    continue;
                }

                let is_outer = self.is_outer_portal(&point);

                // we have found a portal, but portal might be closed:
                // when at the outermost level, only the outer labels AA and ZZ function;
                if is_outer {
                    if item.level == 0
                        && portal != Portal::new('A', 'A')
                        && portal != Portal::new('Z', 'Z')
                    {
                        debug!("skipping portal because we are at the outermost level");
                        continue;
                    }
                    // at any other level, AA and ZZ count as walls, but the other outer labeled tiles bring you one level outward
                    if item.level > 0
                        && (portal == Portal::new('A', 'A') || portal == Portal::new('Z', 'Z'))
                    {
                        debug!("skipping portal because we are NOT at the outermost level");
                        continue;
                    }
                }

                debug!(
                    "walked to {} at {:?}, level {}, is_outer: {}",
                    portal, item.current, item.level, is_outer
                );

                let mut portal_points = self.find_portals(&portal);
                // portal string might be reversed
                if portal_points.is_empty() {
                    portal_points = self.find_portals(&portal.reverse());
                }
                if let Some(other_portal_point) = portal_points.iter().find(|&&p| p != item.current)
                {
                    // go through portal
                    let new_level = match is_outer {
                        // if we've reached outer portal => level - 1
                        true => item.level - 1,
                        // if we've reached inner portal => level + 1
                        false => item.level + 1,
                    };

                    let disc = Discovered {
                        point: *other_portal_point,
                        level: new_level,
                    };
                    if !discovered.contains(&disc) {
                        info!(
                            "going through portal {}: {} -> {}",
                            &portal, item.current, other_portal_point
                        );
                        queue.push_back(IntermediateResult {
                            current: disc.point,
                            level: disc.level,
                            distance: item.distance + 1,
                        });
                        discovered.insert(disc);
                    }
                }
            }
        }
        return best_dist;
    }
}

#[derive(Debug)]
struct IntermediateResult {
    current: Point,
    distance: usize,
    level: usize,
}

impl fmt::Display for IntermediateResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "current: {}, dist: {}, level: {}",
            self.current, self.distance, self.level,
        )
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct Discovered {
    point: Point,
    level: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PathComp {
    portal: Portal,
    level: usize,
    outer: bool,
}

impl fmt::Display for PathComp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({}, {})",
            self.portal,
            self.level,
            if self.outer { "outer" } else { "inner" }
        )
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.row, self.col)
    }
}

pub fn read_input(fname: &str) -> Grid {
    let file = File::open(fname).unwrap();
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents).unwrap();
    return Grid::from_string(&contents);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_outer_portal_test() {
        let grid = read_input("small3.txt");
        assert_eq!(37, grid.row_count());
        assert_eq!(45, grid.col_count());
        assert_eq!(false, grid.is_outer_portal(&Point { row: 9, col: 31 }));
    }
}
