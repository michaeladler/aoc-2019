use std::collections::{HashMap, HashSet, VecDeque};

use log::trace;

use crate::bitset::SmallAsciiBitset;
use crate::grid::{Grid, Tile};

#[derive(Debug)]
pub struct Graph {
    inner: HashMap<Tile, HashSet<Node>>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Node {
    pub label: Tile,
    pub weight: usize,
}

impl Graph {
    pub fn from_grid(grid: &Grid) -> Graph {
        let mut graph = Self::with_size(grid.doors.len() + grid.keys.len() + 1);
        for &src in grid
            .doors
            .iter()
            .chain(grid.keys.iter())
            .chain(grid.entrances.iter())
        {
            let src_tile = grid.tiles[src.0][src.1];
            for (neighb, dist) in grid.edges(src) {
                graph.add_edge(src_tile, grid.tiles[neighb.0][neighb.1], dist);
            }
        }
        return graph;
    }

    pub fn with_size(n: usize) -> Self {
        Self {
            inner: HashMap::with_capacity(n),
        }
    }

    pub fn add_edge(&mut self, from: Tile, to: Tile, weight: usize) {
        trace!("adding edge {:?} -> {:?} with weight {}", from, to, weight);
        self.inner
            .entry(from)
            .or_insert(HashSet::new())
            .insert(Node { label: to, weight });
    }

    // if neighbor is in skip list, use the edges of the neighbors;
    // if we have a key, use the key and go through doors.
    pub fn edges(
        &self,
        start: Tile,
        skip_list: &SmallAsciiBitset,
        keys: &SmallAsciiBitset,
    ) -> HashSet<Node> {
        let mut neighbors = HashSet::with_capacity(2 * self[start].len());

        // BFS
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        visited.insert(start);
        queue.push_back(Node {
            label: start,
            weight: 0,
        });
        trace!(
            "finding edges {:?}, skip_list: {:?}, keys: {:?}",
            start,
            skip_list,
            keys
        );
        while let Some(current) = queue.pop_front() {
            trace!("popped from queue: {:?}", current);
            for node in &self[current.label] {
                trace!("checking {:?}", node);
                if let Some(c) = node.label.char() {
                    if skip_list.contains(c) {
                        trace!("{:?} is in skip_list", node.label);
                        if !visited.contains(&node.label) {
                            trace!("{:?} adding to queue", node.label);
                            visited.insert(node.label);
                            queue.push_back(Node {
                                label: node.label,
                                weight: current.weight + node.weight,
                            });
                        }
                        continue;
                    }
                    trace!("{} not in skip list {:?}", c, skip_list);
                }
                match node.label {
                    Tile::Door(c) => {
                        if keys.contains(door_to_key(c)) {
                            trace!("{:?} is a door but we have the key", node.label);
                            if !visited.contains(&node.label) {
                                trace!("{:?} adding to queue", node.label);
                                visited.insert(node.label);
                                queue.push_back(Node {
                                    label: node.label,
                                    weight: current.weight + node.weight,
                                });
                            }
                        } else {
                            let neighbor_node = Node {
                                label: node.label,
                                weight: current.weight + node.weight,
                            };
                            trace!("adding door to final result: {:?}", neighbor_node);
                            neighbors.insert(neighbor_node);
                        }
                    }
                    Tile::Key(c) => {
                        if node.label != start && !keys.contains(c) {
                            let neighbor_node = Node {
                                label: node.label,
                                weight: current.weight + node.weight,
                            };
                            trace!("adding key to final result: {:?}", neighbor_node);
                            neighbors.insert(neighbor_node);
                        }
                    }
                    _ => {}
                }
            }
        }
        trace!(
            "{:?} has edges {:?}, skip_list: {:?}, keys: {:?}",
            start,
            neighbors,
            skip_list,
            keys
        );
        return neighbors;
    }

    /// return all reachable tiles from start
    pub fn bfs(&self, start: Tile) -> Vec<Tile> {
        let mut result = Vec::new();
        // BFS
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        visited.insert(start);
        queue.push_back(start);
        while let Some(current) = queue.pop_front() {
            for node in &self[current] {
                if !visited.contains(&node.label) {
                    visited.insert(node.label);
                    queue.push_back(node.label);
                    result.push(node.label);
                }
            }
        }
        return result;
    }
}

/// Allows the outgoing edges of a node to be accessed easily.
impl std::ops::Index<Tile> for Graph {
    type Output = HashSet<Node>;
    fn index(&self, index: Tile) -> &Self::Output {
        &self.inner[&index]
    }
}

pub fn door_to_key(door: char) -> char {
    debug_assert!(door.is_ascii());
    debug_assert!(door.is_uppercase());
    return ((door as u8) + 32) as char;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn door_to_key_test() {
        assert_eq!('a', door_to_key('A'));
        assert_eq!('z', door_to_key('Z'));
    }
}
