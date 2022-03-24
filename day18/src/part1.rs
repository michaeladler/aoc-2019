use log::{debug, info};
use std::collections::{HashMap, VecDeque};

use crate::bitset::SmallAsciiBitset;
use crate::graph::Graph;
use crate::grid::{Grid, Tile};

#[derive(Debug, Clone)]
struct IntermediateResult {
    pub current: Tile,
    pub collected_keys: SmallAsciiBitset,
    pub distance: usize,
    pub visited: SmallAsciiBitset,
}

fn find_shortest_path(graph: &Graph, total_keys: usize) -> usize {
    // stores the shortest distances for a tile t and the collected keys until t was reached;
    // this is used to cut off early
    let mut best_paths: HashMap<(Tile, SmallAsciiBitset), usize> = HashMap::new();

    // BFS with cut-off
    let mut queue = VecDeque::new();
    {
        let start = Tile::Entrance('0');
        let mut visited = SmallAsciiBitset::new();
        visited.insert('0');
        let ir = IntermediateResult {
            current: start,
            collected_keys: SmallAsciiBitset::new(),
            distance: 0,
            visited,
        };
        best_paths.insert((start, SmallAsciiBitset::new()), 0);
        queue.push_back(ir);
    }

    let mut best_result = usize::MAX;
    // still some keys left; pick each robot and move it
    debug!("total_keys: {}", total_keys);
    while let Some(ir) = queue.pop_front() {
        let key_count = ir.collected_keys.len();
        debug!(
            "popped item from queue. key count: {}, queue size: {}",
            key_count,
            queue.len()
        );
        if key_count == total_keys {
            let total_dist = ir.distance;
            debug!("found all keys, distance: {}", total_dist);
            if total_dist < best_result {
                best_result = total_dist;
                info!("new best result: {}", total_dist);
            }
            continue;
        }

        for node in graph
            .edges(ir.current, &ir.visited, &ir.collected_keys)
            .iter()
            .filter(|node| !node.label.is_door())
        {
            let p = node.label;
            let dist = node.weight;
            debug!(
                ">> visiting {:?} -> {:?}, dist: {}, visited: {:?}, keys: {:?}",
                ir.current, p, dist, ir.visited, ir.collected_keys,
            );
            let distance = ir.distance + dist;
            if distance > best_result {
                debug!(
                    "Cutting off branch, distance too large: {} > {}",
                    distance, best_result
                );
                continue;
            }
            if let Some(&current_best_for_p) = best_paths.get(&(p, ir.collected_keys)) {
                if current_best_for_p < distance {
                    debug!("Cutting off branch: {} < {}", current_best_for_p, distance);
                    continue;
                }
            }
            best_paths.insert((p, ir.collected_keys), distance);
            let mut collected_keys = ir.collected_keys;
            if let Tile::Key(c) = p {
                collected_keys.insert(c);
            }
            let mut new_visited = ir.visited;
            new_visited.insert(p.char().unwrap());
            let new_ir = IntermediateResult {
                current: p,
                collected_keys,
                distance,
                visited: new_visited,
            };
            queue.push_back(new_ir);
        }
    }

    return best_result;
}

pub fn part1(fname: &str) -> usize {
    let grid = Grid::from_file(fname);
    let total_keys = grid.keys.len();
    let graph = Graph::from_grid(&grid);
    return find_shortest_path(&graph, total_keys);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // slow
    fn part1_small_test() {
        assert_eq!(136, part1(&"small.txt"[..]));
    }

    #[test]
    fn part1_small2_test() {
        assert_eq!(86, part1(&"small2.txt"[..]));
    }

    #[test]
    fn part1_small3_test() {
        assert_eq!(132, part1(&"small3.txt"[..]));
    }

    #[test]
    fn part1_small4_test() {
        assert_eq!(81, part1(&"small4.txt"[..]));
    }

    #[test]
    // slow
    fn part1_test() {
        assert_eq!(4620, part1(&"input.txt"[..]));
    }
}
