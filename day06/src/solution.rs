use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use log::debug;

fn read_input(fname: &str, directed: bool) -> HashMap<String, Vec<String>> {
    let file = File::open(fname).unwrap();
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents).unwrap();
    let mut result = HashMap::new();
    for line in contents.lines() {
        let parts: Vec<&str> = line.split(')').collect();
        let entry = result.entry(parts[0].to_string()).or_insert(Vec::new());
        entry.push(parts[1].to_string());
        if !directed {
            let entry = result.entry(parts[1].to_string()).or_insert(Vec::new());
            entry.push(parts[0].to_string());
        }
    }
    return result;
}

fn bfs(orbits: &HashMap<String, Vec<String>>, start: &str) -> HashMap<String, usize> {
    let mut queue = VecDeque::new();
    let mut discovered = HashSet::new();
    queue.push_back((start.to_string(), 0));
    discovered.insert(start.to_string());

    let mut distances = HashMap::new();
    distances.insert(start.to_string(), 0);

    while let Some((current, dist)) = queue.pop_front() {
        debug!("visiting {}", current);
        if let Some(neighbors) = orbits.get(&current) {
            let new_dist = dist + 1;
            for nb in neighbors.iter() {
                debug!("{} has neighbor {}", current, nb);
                if !discovered.contains(nb) {
                    let t = nb.clone();
                    discovered.insert(t.clone());
                    distances.insert(t.clone(), new_dist);
                    debug!("adding {} to queue", &t);
                    queue.push_back((t, new_dist));
                }
            }
        }
    }
    return distances;
}

pub fn part1(fname: &str) -> usize {
    let orbits = read_input(fname, true);
    let distances = bfs(&orbits, &"COM"[..]);
    return distances.values().sum();
}

pub fn part2(fname: &str) -> usize {
    let orbits = read_input(fname, false);
    let distances = bfs(&orbits, &"YOU"[..]);
    return distances[&"SAN"[..]] - 2;
}
