use log::{debug, info};
use std::collections::{HashMap, VecDeque};

use crate::bitset::SmallAsciiBitset;
use crate::graph::{door_to_key, Graph, Node};
use crate::grid::{Grid, Tile};

const ENTRANCE_COUNT: u8 = 4;

#[derive(Debug, Clone)]
struct Robots {
    results: [IntermediateResult; ENTRANCE_COUNT as usize],
}

impl Robots {
    pub fn visit(
        &self,
        robot_id: usize,
        node: &Node,
        best_paths: &mut HashMap<(char, SmallAsciiBitset), usize>,
        best_result: usize,
    ) -> Option<Self> {
        let ir = &self.results[robot_id];
        let dist = node.weight;
        debug!(
            ">> Robot {} walking from {:?} to {:?}, dist: {} ",
            robot_id, ir.current, node.label, dist
        );
        let distance = ir.distance + dist;
        if distance > best_result {
            debug!(
                "Cutting off branch, distance too large: {} > {}",
                distance, best_result
            );
            return None;
        }

        if let Tile::Key(key) = node.label {
            if let Some(&current_best_for_p) = best_paths.get(&(key, ir.collected_keys)) {
                if current_best_for_p < distance {
                    debug!("Cutting off branch: {} < {}", current_best_for_p, distance);
                    return None;
                }
            }

            best_paths.insert((key, ir.collected_keys), distance);
        }

        let mut new_result = self.clone();
        new_result.results[robot_id].visit(node);
        // add key to other robots
        if let Tile::Key(c) = node.label {
            for j in 0..ENTRANCE_COUNT as usize {
                if j != robot_id {
                    new_result.results[j].collected_keys.insert(c);
                }
            }
        }
        return Some(new_result);
    }
}

#[derive(Debug, Clone)]
struct IntermediateResult {
    pub collected_keys: SmallAsciiBitset,
    pub visited: SmallAsciiBitset,
    pub current: Tile,
    pub distance: usize,
    pub force_next: Option<Node>,
}

impl IntermediateResult {
    pub fn new(start: char) -> Self {
        Self {
            current: Tile::Entrance(start),
            collected_keys: SmallAsciiBitset::new(),
            visited: SmallAsciiBitset::from(start),
            distance: 0,
            force_next: None,
        }
    }

    pub fn visit(&mut self, node: &Node) {
        self.current = node.label;
        match node.label {
            Tile::Key(c) => {
                self.collected_keys.insert(c);
                self.visited.insert(c);
            }
            Tile::Door(c) => {
                self.visited.insert(c);
            }
            _ => {
                panic!("Unexpected tile")
            }
        }
        self.distance += node.weight;
        self.force_next = None;
    }
}

impl Robots {
    fn count_collected_keys(&self) -> usize {
        let mut all_keys = SmallAsciiBitset::new();
        for r in &self.results {
            all_keys.union(&r.collected_keys);
        }
        return all_keys.len();
    }

    fn total_distance(&self) -> usize {
        self.results.iter().map(move |x| x.distance).sum()
    }
}

pub fn find_shortest_path(graph: &Graph, total_keys: usize) -> usize {
    // stores the shortest distances for a tile t and the collected keys until t was reached;
    // this is used to cut off early
    let mut best_paths: HashMap<(char, SmallAsciiBitset), usize> = HashMap::new();

    // for each robot, store what it can encounter
    let mut expected_keys: HashMap<u8, SmallAsciiBitset> = HashMap::with_capacity(4);
    let mut expected_doors: HashMap<u8, SmallAsciiBitset> = HashMap::with_capacity(4);

    for i in 0..ENTRANCE_COUNT {
        let mut keys = SmallAsciiBitset::new();
        let mut doors = SmallAsciiBitset::new();
        for t in &graph.bfs(Tile::Entrance(('0' as u8 + i) as char)) {
            match t {
                Tile::Door(c) => {
                    doors.insert(*c);
                }
                Tile::Key(c) => {
                    keys.insert(*c);
                }
                _ => {}
            }
        }
        expected_keys.insert(i, keys);
        expected_doors.insert(i, doors);
    }

    // BFS with cut-off
    let mut queue = VecDeque::with_capacity(4096);
    queue.push_back(Robots {
        results: [
            IntermediateResult::new('0'),
            IntermediateResult::new('1'),
            IntermediateResult::new('2'),
            IntermediateResult::new('3'),
        ],
    });

    let mut best_result = usize::MAX;
    while let Some(mut robot_result) = queue.pop_front() {
        debug!(
            ">> popped item from queue. keys left: {}, current best: {}, queue size: {}",
            total_keys - robot_result.count_collected_keys(),
            best_result,
            queue.len()
        );

        // avoid branching as much as possible (exponential growth)
        // if there is only one choice, just go there
        loop {
            let mut found_canonical_choice = false;
            for i in 0..(ENTRANCE_COUNT as usize) {
                let ir = &robot_result.results[i];
                let edges = graph.edges(ir.current, &ir.visited, &ir.collected_keys);
                if edges.len() == 1 {
                    let node = edges.iter().next().unwrap();
                    // TODO: if it's a door, we can also deal with it here
                    if let Tile::Key(c) = node.label {
                        if let Some(new_robot_result) =
                            robot_result.visit(i, node, &mut best_paths, best_result)
                        {
                            debug!("Robot {} had no choice but to visit key {:?}", i, c);
                            found_canonical_choice = true;
                            robot_result = new_robot_result;
                        }
                    }
                }
            }
            if !found_canonical_choice {
                break;
            }
        }

        if robot_result.count_collected_keys() == total_keys {
            let total_dist = robot_result.total_distance();
            debug!("found all keys, distance: {}", total_dist);
            if total_dist < best_result {
                best_result = total_dist;
                info!("new best result: {}", total_dist);
            }
            continue;
        }

        // more than one choice, we need to branch out
        for i in 0..(ENTRANCE_COUNT as usize) {
            let ir = &robot_result.results[i];
            // no need to move this robot if it already has all keys in its (sub)grid
            if ir.collected_keys == expected_keys[&(i as u8)] {
                info!("Robot {} already found all keys, not moving it anymore", i);
                continue;
            }

            if let Some(node) = &ir.force_next {
                let c = node.label.char().unwrap(); // must be door
                let key = door_to_key(c);
                // do we have the key now?
                if ir.collected_keys.contains(key) {
                    info!("forced to visit {} now", key);
                    let mut new_robot_result = robot_result.clone();
                    new_robot_result.results[i].visit(node);
                    queue.push_back(new_robot_result);
                } else {
                    debug!("cannot go there yet {}", c);
                    queue.push_back(robot_result.clone());
                }
                continue;
            }

            let edges = graph.edges(ir.current, &ir.visited, &ir.collected_keys);
            for node in &edges {
                match node.label {
                    Tile::Door(c) => {
                        // are we blocked because some other robot still needs to find the key?
                        // if so, we also have to wait for the robot and then try to follow this path
                        for j in 0..(ENTRANCE_COUNT as usize) {
                            if j != i && expected_doors[&(j as u8)].contains(c) {
                                debug!("robot {} door {} is blocked because robot {} still needs to find its key", i, c, j);
                                let mut new_robot_result = robot_result.clone();
                                new_robot_result.results[i].force_next = Some(node.clone());
                                queue.push_back(new_robot_result);
                            }
                        }
                    }
                    Tile::Key(_) => {
                        if let Some(new_robot_result) =
                            robot_result.visit(i, node, &mut best_paths, best_result)
                        {
                            queue.push_back(new_robot_result);
                        }
                    }
                    _ => {
                        panic!("Unexpected edge");
                    }
                };
            }
        }
    }

    return best_result;
}

pub fn part2(fname: &str) -> usize {
    let mut grid = Grid::from_file(fname);
    grid.to_many_worlds();
    debug!("{}", grid);
    let graph = Graph::from_grid(&grid);
    return find_shortest_path(&graph, grid.keys.len());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn part2_small_test() {
        assert_eq!(8, part2(&"part2_small.txt"));
    }

    #[test]
    fn part2_small2_test() {
        assert_eq!(24, part2(&"part2_small2.txt"));
    }

    #[test]
    fn part2_small3_test() {
        assert_eq!(32, part2(&"part2_small3.txt"));
    }

    #[test]
    // slow
    fn part2_test() {
        assert_eq!(1564, part2(&"input.txt"[..]));
    }
}
