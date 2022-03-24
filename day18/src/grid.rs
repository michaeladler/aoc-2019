use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Tile {
    Wall,
    Open,
    Key(char),
    Door(char),
    Entrance(char),
}

impl Tile {
    /// Numeric encodes the Tile to a unique u8 (ASCII).
    pub fn char(&self) -> Option<char> {
        use Tile::*;
        match self {
            Key(c) => Some(*c),
            Door(c) => Some(*c),
            Entrance(c) => Some(*c),
            _ => None,
        }
    }

    pub fn is_door(&self) -> bool {
        if let Tile::Door(_) = self {
            return true;
        }
        return false;
    }

}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Tile::Wall => write!(f, "#"),
            Tile::Open => write!(f, "."),
            Tile::Key(c) => write!(f, "{}", c),
            Tile::Door(c) => write!(f, "{}", c),
            Tile::Entrance(d) => write!(f, "{}", d),
        }
    }
}

type Point = (usize, usize);

#[derive(Debug)]
pub struct Grid {
    // first dimension is always row, second dim is column
    pub tiles: Vec<Vec<Tile>>,
    pub doors: Vec<Point>,
    pub keys: Vec<Point>,
    pub entrances: Vec<Point>,
}

impl Grid {
    pub fn from_file(fname: &str) -> Self {
        let file = File::open(fname).unwrap();
        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();
        buf_reader.read_to_string(&mut contents).unwrap();
        return Self::from_string(&contents);
    }

    pub fn from_string(data: &str) -> Grid {
        let mut tiles = Vec::with_capacity(81);
        let mut doors = Vec::new();
        let mut keys = Vec::new();
        let mut entrance = None;

        for (row, line) in data.split('\n').enumerate() {
            let mut v = Vec::with_capacity(81);
            for (col, c) in line.chars().enumerate() {
                v.push(match c {
                    '#' => Tile::Wall,
                    '.' => Tile::Open,
                    '@' => {
                        entrance.replace((row, col));
                        Tile::Entrance('0')
                    }
                    'a'..='z' => {
                        keys.push((row, col));
                        Tile::Key(c)
                    }
                    'A'..='Z' => {
                        doors.push((row, col));
                        Tile::Door(c)
                    }
                    _ => panic!("Unexpected input: {}", c),
                });
            }
            if !v.is_empty() {
                tiles.push(v);
            }
        }
        return Grid {
            tiles,
            doors,
            keys,
            entrances: vec![entrance.unwrap()],
        };
    }

    /// Find outgoing edges from the given `start` point
    pub fn edges(&self, start: Point) -> HashMap<Point, usize> {
        let mut result = HashMap::new();
        // perform a small BFS

        let mut visited = HashSet::new();
        visited.insert(start);

        let mut queue = VecDeque::with_capacity(32);
        queue.push_back(start);

        let mut distances = HashMap::new();
        distances.insert(start, 0);

        while let Some(p) = queue.pop_front() {
            for neighb in self.neighbors(p) {
                if !visited.contains(&neighb) {
                    visited.insert(neighb);
                    let dist = distances[&p] + 1;
                    distances.insert(neighb, dist);
                    if self.get(neighb) == Some(Tile::Open) {
                        queue.push_back(neighb);
                    } else {
                        result.insert(neighb, dist);
                    }
                }
            }
        }
        return result;
    }

    pub fn to_many_worlds(&mut self) {
        assert_eq!(1, self.entrances.len()); // do not call this method twice!
        let (row, col) = self.entrances[0];
        self.tiles[row][col] = Tile::Wall;

        for p in &[
            (row + 1, col),
            (row - 1, col),
            (row, col + 1),
            (row, col - 1),
        ] {
            assert_eq!(Tile::Open, self.tiles[p.0][p.1]);
            self.tiles[p.0][p.1] = Tile::Wall;
        }

        self.entrances.clear();
        let mut i = '0';
        for p in &[
            (row + 1, col + 1),
            (row - 1, col - 1),
            (row + 1, col - 1),
            (row - 1, col + 1),
        ] {
            assert_eq!(Tile::Open, self.tiles[p.0][p.1]);
            self.tiles[p.0][p.1] = Tile::Entrance(i);
            self.entrances.push(*p);
            i = ((i as u8) + 1) as char;
        }
    }

    fn get(&self, (row_idx, col_idx): Point) -> Option<Tile> {
        if let Some(col) = self.tiles.get(row_idx) {
            if let Some(tile) = col.get(col_idx) {
                return Some(*tile);
            }
        }
        None
    }

    /// Get all neighbor points (N, E, S, W) which are _not_ a wall.
    fn neighbors(&self, (row_idx, col_idx): Point) -> Vec<Point> {
        let mut result = Vec::new();
        let candidates = [
            (row_idx - 1, col_idx),
            (row_idx + 1, col_idx),
            (row_idx, col_idx - 1),
            (row_idx, col_idx + 1),
        ];
        for c in &candidates {
            if let Some(tile) = self.get(*c) {
                if tile != Tile::Wall {
                    result.push(*c);
                }
            }
        }
        return result;
    }
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Grid:\n")?;
        for row in &self.tiles {
            for tile in row {
                write!(f, "{}", tile)?;
            }
            write!(f, "\n")?;
        }
        return Ok(());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_edges_test_small() {
        let grid = Grid::from_file(&"small.txt"[..]);
        assert_eq!(vec![(4, 8)], grid.entrances);
        let actual_distances = grid.edges(grid.entrances[0]);
        assert_eq!(8, actual_distances.keys().len());
        let expected = vec![
            (1, 6),
            (3, 6),
            (5, 6),
            (7, 6),
            (1, 10),
            (3, 10),
            (5, 10),
            (7, 10),
        ];
        for p in &expected {
            assert!(actual_distances.contains_key(p));
        }
    }

    #[test]
    fn grid_edges_test_small4() {
        let grid = Grid::from_file(&"small4.txt"[..]);
        assert_eq!(vec![(1, 1)], grid.entrances);
        let key_a = (1, 16);
        let actual_distances = grid.edges(key_a);
        assert_eq!(5, actual_distances.keys().len());
    }
}
