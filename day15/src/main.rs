extern crate aoc2019;

#[macro_use]
extern crate log;
extern crate env_logger;
extern crate im_rc;

use im_rc::vector::Vector;
use std::collections::{HashMap, HashSet, VecDeque};

use aoc2019::intcode::IntcodeProgram;

#[derive(Debug, PartialEq, Clone, Copy)]
enum Direction {
    North,
    South,
    West,
    East,
}

impl Direction {
    pub fn value(&self) -> i64 {
        match self {
            Direction::North => 1,
            Direction::South => 2,
            Direction::West => 3,
            Direction::East => 4,
        }
    }

    pub fn from_points(src: &(i64, i64), dst: &(i64, i64)) -> Option<Direction> {
        let dx = dst.0 - src.0;
        let dy = dst.1 - src.1;
        let direction = match (dx, dy) {
            (0, -1) => Some(Direction::North),
            (0, 1) => Some(Direction::South),
            (1, 0) => Some(Direction::East),
            (-1, 0) => Some(Direction::West),
            _ => None,
        };
        trace!(
            "Calculated direction from {:?} to {:?}: {:?}",
            src,
            dst,
            direction
        );
        return direction;
    }

    pub fn opposite(&self) -> Direction {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
            Direction::East => Direction::West,
        }
    }
}

#[derive(Debug, PartialEq)]
enum DroidResult {
    Blocked,
    Progress,
    Final,
}

impl DroidResult {
    pub fn from_value(value: i64) -> DroidResult {
        match value {
            0 => DroidResult::Blocked,
            1 => DroidResult::Progress,
            2 => DroidResult::Final,
            _ => panic!("Unsupported movement value: {}", value),
        }
    }
}

#[derive(Debug)]
struct Droid {
    program: IntcodeProgram,
    path: Vec<Direction>,
    pos: (i64, i64),
}

impl Droid {
    pub fn new() -> Self {
        let code = vec![
            3, 1033, 1008, 1033, 1, 1032, 1005, 1032, 31, 1008, 1033, 2, 1032, 1005, 1032, 58,
            1008, 1033, 3, 1032, 1005, 1032, 81, 1008, 1033, 4, 1032, 1005, 1032, 104, 99, 1002,
            1034, 1, 1039, 1002, 1036, 1, 1041, 1001, 1035, -1, 1040, 1008, 1038, 0, 1043, 102, -1,
            1043, 1032, 1, 1037, 1032, 1042, 1106, 0, 124, 1001, 1034, 0, 1039, 1002, 1036, 1,
            1041, 1001, 1035, 1, 1040, 1008, 1038, 0, 1043, 1, 1037, 1038, 1042, 1106, 0, 124,
            1001, 1034, -1, 1039, 1008, 1036, 0, 1041, 1002, 1035, 1, 1040, 1001, 1038, 0, 1043,
            101, 0, 1037, 1042, 1105, 1, 124, 1001, 1034, 1, 1039, 1008, 1036, 0, 1041, 102, 1,
            1035, 1040, 1001, 1038, 0, 1043, 101, 0, 1037, 1042, 1006, 1039, 217, 1006, 1040, 217,
            1008, 1039, 40, 1032, 1005, 1032, 217, 1008, 1040, 40, 1032, 1005, 1032, 217, 1008,
            1039, 39, 1032, 1006, 1032, 165, 1008, 1040, 3, 1032, 1006, 1032, 165, 1102, 1, 2,
            1044, 1106, 0, 224, 2, 1041, 1043, 1032, 1006, 1032, 179, 1102, 1, 1, 1044, 1106, 0,
            224, 1, 1041, 1043, 1032, 1006, 1032, 217, 1, 1042, 1043, 1032, 1001, 1032, -1, 1032,
            1002, 1032, 39, 1032, 1, 1032, 1039, 1032, 101, -1, 1032, 1032, 101, 252, 1032, 211,
            1007, 0, 59, 1044, 1105, 1, 224, 1102, 1, 0, 1044, 1105, 1, 224, 1006, 1044, 247, 101,
            0, 1039, 1034, 1001, 1040, 0, 1035, 101, 0, 1041, 1036, 1002, 1043, 1, 1038, 1002,
            1042, 1, 1037, 4, 1044, 1105, 1, 0, 93, 27, 71, 56, 88, 17, 30, 78, 5, 57, 79, 56, 3,
            82, 62, 58, 16, 2, 21, 89, 95, 33, 12, 32, 90, 12, 7, 76, 83, 31, 8, 13, 27, 89, 60,
            33, 7, 40, 22, 50, 8, 63, 35, 45, 57, 94, 81, 4, 65, 33, 47, 73, 28, 98, 11, 70, 95,
            17, 82, 39, 19, 73, 62, 56, 80, 85, 23, 91, 39, 86, 91, 82, 50, 37, 86, 4, 90, 83, 8,
            65, 56, 63, 15, 99, 51, 3, 60, 60, 77, 58, 90, 82, 5, 52, 14, 87, 37, 74, 85, 43, 17,
            61, 91, 35, 31, 81, 19, 12, 34, 54, 9, 66, 34, 69, 67, 21, 4, 14, 87, 22, 76, 26, 82,
            79, 4, 69, 48, 73, 8, 73, 57, 61, 83, 23, 83, 60, 3, 41, 75, 67, 53, 44, 91, 27, 52,
            84, 66, 13, 65, 95, 81, 83, 30, 26, 60, 12, 33, 92, 81, 46, 78, 25, 13, 72, 87, 26, 63,
            57, 35, 2, 60, 96, 63, 26, 2, 76, 95, 21, 38, 60, 5, 79, 86, 89, 47, 42, 12, 91, 30,
            52, 69, 55, 67, 73, 47, 44, 5, 86, 8, 52, 69, 81, 23, 70, 3, 38, 41, 89, 88, 58, 41, 9,
            96, 27, 67, 21, 14, 68, 67, 35, 84, 23, 20, 91, 63, 47, 75, 34, 70, 57, 13, 54, 82, 33,
            61, 27, 97, 88, 46, 44, 56, 74, 14, 5, 96, 71, 16, 40, 86, 61, 84, 41, 81, 81, 16, 88,
            51, 41, 96, 76, 28, 97, 44, 41, 65, 87, 50, 73, 58, 71, 46, 73, 51, 43, 18, 46, 99, 74,
            65, 9, 89, 3, 77, 22, 34, 93, 94, 39, 54, 96, 12, 35, 62, 87, 56, 69, 64, 9, 34, 91,
            64, 71, 28, 10, 94, 1, 96, 20, 67, 92, 39, 37, 26, 79, 68, 16, 76, 57, 83, 92, 46, 75,
            99, 26, 64, 39, 72, 65, 37, 93, 65, 5, 53, 62, 36, 13, 97, 14, 38, 85, 33, 76, 56, 99,
            29, 64, 84, 28, 19, 91, 92, 55, 33, 88, 32, 70, 38, 53, 76, 1, 76, 35, 26, 75, 18, 18,
            7, 88, 19, 53, 65, 22, 91, 20, 85, 15, 13, 72, 82, 13, 31, 75, 62, 68, 4, 56, 91, 89,
            56, 10, 46, 63, 7, 74, 50, 15, 85, 87, 64, 77, 12, 95, 10, 66, 77, 51, 6, 61, 75, 91,
            75, 85, 61, 78, 4, 97, 99, 4, 90, 34, 89, 44, 44, 68, 89, 30, 20, 70, 24, 22, 81, 22,
            77, 61, 33, 89, 2, 11, 75, 50, 85, 13, 43, 56, 78, 73, 49, 27, 38, 78, 56, 90, 17, 94,
            72, 51, 5, 55, 67, 32, 19, 81, 81, 45, 83, 18, 96, 33, 75, 53, 4, 29, 87, 80, 33, 57,
            78, 80, 43, 68, 57, 71, 83, 10, 18, 98, 70, 36, 61, 31, 73, 33, 69, 24, 78, 76, 43, 88,
            96, 16, 14, 91, 43, 66, 15, 98, 44, 48, 68, 57, 72, 48, 49, 89, 62, 31, 55, 83, 68, 86,
            97, 16, 25, 87, 13, 74, 40, 82, 43, 48, 85, 40, 45, 72, 33, 60, 84, 4, 47, 96, 19, 92,
            75, 73, 46, 6, 69, 4, 81, 98, 89, 48, 55, 89, 24, 64, 31, 47, 50, 93, 72, 47, 72, 36,
            79, 7, 24, 66, 60, 65, 18, 81, 93, 40, 37, 36, 62, 94, 48, 8, 77, 21, 82, 22, 65, 20,
            46, 85, 47, 52, 70, 55, 74, 19, 65, 15, 72, 81, 57, 67, 46, 94, 21, 16, 94, 84, 36, 43,
            62, 82, 48, 47, 79, 5, 96, 39, 58, 85, 80, 31, 7, 98, 23, 69, 22, 99, 37, 69, 35, 66,
            36, 70, 3, 69, 47, 6, 64, 38, 69, 42, 57, 91, 89, 21, 89, 13, 42, 78, 24, 44, 79, 74,
            65, 63, 85, 10, 50, 71, 94, 26, 78, 55, 5, 26, 71, 46, 20, 83, 96, 51, 87, 2, 99, 83,
            5, 38, 86, 8, 13, 94, 61, 93, 39, 67, 23, 60, 74, 87, 57, 30, 72, 23, 19, 95, 57, 93,
            83, 58, 34, 83, 35, 4, 47, 81, 88, 24, 87, 34, 93, 79, 70, 18, 24, 73, 98, 76, 77, 24,
            93, 18, 66, 56, 87, 25, 29, 7, 7, 97, 40, 61, 56, 96, 96, 1, 42, 21, 92, 73, 11, 10,
            97, 69, 58, 93, 2, 82, 27, 96, 7, 84, 44, 67, 57, 63, 13, 79, 56, 72, 34, 89, 26, 94,
            24, 86, 99, 71, 73, 98, 26, 89, 10, 98, 5, 64, 70, 85, 32, 61, 35, 67, 0, 0, 21, 21, 1,
            10, 1, 0, 0, 0, 0, 0, 0,
        ];

        let program = IntcodeProgram::new(code);
        return Self {
            program,
            path: Vec::new(),
            pos: (0, 0),
        };
    }

    pub fn walk(&mut self, direction: Direction) -> DroidResult {
        let mut output = Vec::new();
        let status = self.program.run(&mut vec![direction.value()], &mut output);
        trace!("Program status: {:?}", status);
        debug_assert_eq!(output.len(), 1);
        let result = DroidResult::from_value(output[0]);
        match result {
            DroidResult::Progress | DroidResult::Final => {
                self.path.push(direction);
                self.update_position(direction);
            }
            _ => {
                debug!(
                    "Droid at {:?} tried to move {:?} but failed",
                    self.pos, direction
                );
            }
        }
        return result;
    }

    pub fn backtrack(&mut self) -> bool {
        debug!("Starting Backtracking");
        let result = match self.path.pop() {
            Some(direction) => {
                let move_result = self.walk(direction.opposite());
                debug_assert_ne!(move_result, DroidResult::Blocked);
                // pretend we didn't move
                self.path.pop();
                true
            }
            None => false,
        };
        debug!("Finished Backtracking with result: {}", result);
        return result;
    }

    fn update_position(&mut self, direction: Direction) {
        let old_x = self.pos.0;
        let old_y = self.pos.1;
        let new_pos = match direction {
            Direction::North => (old_x, old_y - 1),
            Direction::South => (old_x, old_y + 1),
            Direction::East => (old_x + 1, old_y),
            Direction::West => (old_x - 1, old_y),
        };
        debug!(
            "Droid moved {:?}: ({}, {}) -> {:?}",
            direction, old_x, old_y, new_pos
        );
        self.pos = new_pos;
    }
}

#[derive(Debug, PartialEq, Clone)]
struct Node {
    pos: (i64, i64),
    dist: usize,
}

#[derive(Debug)]
struct Graph {
    map: HashMap<(i64, i64), HashSet<(i64, i64)>>,
}

#[derive(Debug, Clone)]
struct Edge {
    pos: (i64, i64),
    path: Vector<(i64, i64)>,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
    pub fn find_path(&self, src: (i64, i64), dst: (i64, i64)) -> Vector<(i64, i64)> {
        debug!("Need to find path from {:?} to {:?}", src, dst);

        let mut stack: Vec<Edge> = Vec::new();
        let mut discovered = HashSet::new();
        let mut initial_path = Vector::new();
        initial_path.push_back(src);
        stack.push(Edge {
            pos: src,
            path: initial_path,
        });

        while let Some(edge) = stack.pop() {
            debug!("Processing {:?}", edge);
            if !discovered.contains(&edge.pos) {
                discovered.insert(edge.pos);
                if let Some(children) = self.map.get(&edge.pos) {
                    debug!("Edge has children: {:?}", children);
                    for child in children.iter() {
                        debug!("Processing child: {:?}", child);
                        let mut path = edge.path.clone();
                        path.push_back(*child);
                        if *child == dst {
                            debug!("Found dst! path={:?}", path);
                            return path;
                        }
                        stack.push(Edge {
                            pos: *child,
                            path: path,
                        });
                    }
                }
            }
        }
        return Vector::new();
    }

    pub fn upsert(&mut self, parent: (i64, i64), child: (i64, i64)) {
        let children = self.map.entry(parent).or_insert(HashSet::new());
        children.insert(child);
    }
}

fn solve() -> (usize, usize) {
    let mut droid = Droid::new();

    let mut graph = Graph::new();
    let mut visited: HashSet<(i64, i64)> = HashSet::new();

    // BFS algorithm
    let mut queue: VecDeque<Node> = VecDeque::new();
    queue.push_back(Node {
        pos: (0, 0),
        dist: 0,
    });
    visited.insert((0, 0));

    let mut oxygen_pos = None;
    let mut min_dist = std::usize::MAX;
    while let Some(node) = queue.pop_front() {
        debug!("*** Popped node: {:?}", node);
        if droid.pos != node.pos {
            debug!("Need to move droid from {:?} to {:?}", droid.pos, node.pos);
            let path = graph.find_path(droid.pos, node.pos);
            let mut prev = droid.pos;
            for i in 1..path.len() {
                let p = path[i];
                let direction = Direction::from_points(&prev, &p).expect("Path must be routable");
                let status = droid.walk(direction);
                debug_assert_ne!(status, DroidResult::Blocked);
                prev = p;
            }
        }

        debug_assert_eq!(droid.pos, node.pos);
        debug!("Droid exploring surrounding area");
        for direction in &[
            Direction::North,
            Direction::South,
            Direction::East,
            Direction::West,
        ] {
            let status = droid.walk(*direction);
            let dist = node.dist + 1;
            match status {
                DroidResult::Final | DroidResult::Progress => {
                    graph.upsert(node.pos, droid.pos);
                    if !visited.contains(&droid.pos) {
                        visited.insert(droid.pos);
                        queue.push_back(Node {
                            pos: droid.pos,
                            dist: dist,
                        });
                    }
                    debug!("Reached target");
                    if status == DroidResult::Final && dist < min_dist {
                        min_dist = dist;
                        oxygen_pos = Some(droid.pos);
                    }
                    droid.backtrack();
                }
                _ => {
                    debug!("Blocked");
                }
            }
        }
        trace!("Queue: {:?}", queue);
        trace!("Graph: {:?}", graph);
    }

    println!(
        "Oxygen tank is located at {:?} and takes {} steps to reach",
        oxygen_pos, min_dist
    );
    let part1 = min_dist;

    info!("Visited: {:?}", visited);
    info!("Starting oxygen simulation");
    let mut oxygen_cells = Vector::new();
    let mut steps = 0;

    oxygen_cells.push_back(oxygen_pos.expect("Oxygen must exist"));
    loop {
        let mut new_cells = Vector::new();
        for cell in oxygen_cells.iter() {
            info!("Spreading oxygen from {:?}", cell);
            let x = cell.0;
            let y = cell.1;
            for candidate in &[(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)] {
                if visited.remove(&candidate) {
                    info!("Candidate filled with oxygen: {:?}", candidate);
                    new_cells.push_back(*candidate);
                }
            }
        }
        if new_cells.is_empty() {
            break;
        }
        oxygen_cells.append(new_cells);
        steps += 1;
    }
    let part2 = steps;

    return (part1, part2);
}

fn main() {
    env_logger::init();
    let (part1, part2) = solve();
    println!("Part One: {}", part1);
    println!("Part Two: {}", part2);
}

#[test]
fn solve_test() {
    let (part1, part2) = solve();
    assert_eq!(224, part1);
    assert_eq!(284, part2);
}
