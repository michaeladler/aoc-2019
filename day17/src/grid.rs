use std::fmt;

#[derive(Debug)]
pub struct Grid {
    pub fields: Vec<Vec<Field>>,
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Field {
    Tile,
    Empty,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

impl Grid {
    fn point(&self, x: i64, y: i64) -> Option<Field> {
        if x < 0 || y < 0 {
            return None;
        }
        if let Some(row) = self.fields.get(y as usize) {
            return row.get(x as usize).map(|x| *x);
        }
        return None;
    }

    fn horizontal_neighbors(&self, x: i64, y: i64) -> usize {
        let mut count = 0;
        if Some(Field::Tile) == self.point(x + 1, y) {
            count = count + 1;
        }
        if Some(Field::Tile) == self.point(x - 1, y) {
            count = count + 1;
        }
        return count;
    }

    fn vertical_neighbors(&self, x: i64, y: i64) -> usize {
        let mut count = 0;
        if Some(Field::Tile) == self.point(x, y + 1) {
            count = count + 1;
        }
        if Some(Field::Tile) == self.point(x, y - 1) {
            count = count + 1;
        }
        return count;
    }

    fn is_intersection(&self, x: i64, y: i64) -> bool {
        return self.point(x, y) == Some(Field::Tile)
            && self.horizontal_neighbors(x, y) == 2
            && self.vertical_neighbors(x, y) == 2;
    }

    pub fn intersections(&self) -> Vec<(usize, usize)> {
        let mut result = Vec::new();
        for y in 0..self.height {
            for x in 0..self.width {
                if self.is_intersection(x as i64, y as i64) {
                    result.push((x, y));
                }
            }
        }
        return result;
    }

    fn walk_horizontal(&self, sign: i64, x: i64, y: i64) -> (i64, i64) {
        let mut delta = 1;
        let mut last_ok = (x, y);
        while delta < self.width as i64 {
            let mut has_update = false;
            if let Some(row) = self.fields.get(y as usize) {
                let new_x = x + sign * delta;
                if new_x >= 0 {
                    if let Some(Field::Tile) = row.get(new_x as usize) {
                        last_ok = (new_x, y);
                        has_update = true;
                    }
                }
            }
            if !has_update {
                return last_ok;
            }
            delta = delta + 1;
        }
        return last_ok;
    }

    pub fn walk_east(&self, x: i64, y: i64) -> (i64, i64) {
        self.walk_horizontal(1, x, y)
    }

    pub fn walk_west(&self, x: i64, y: i64) -> (i64, i64) {
        self.walk_horizontal(-1, x, y)
    }

    fn walk_vertical(&self, sign: i64, x: i64, y: i64) -> (i64, i64) {
        let mut delta = 1;
        let mut last_ok = (x, y);
        while delta < self.height as i64 {
            let mut has_update = false;
            let new_y = y + sign * delta;
            if new_y >= 0 {
                if let Some(row) = self.fields.get(new_y as usize) {
                    if let Some(Field::Tile) = row.get(x as usize) {
                        last_ok = (x, new_y);
                        has_update = true;
                    }
                }
            }
            if !has_update {
                return last_ok;
            }
            delta = delta + 1;
        }
        return last_ok;
    }

    pub fn walk_south(&self, x: i64, y: i64) -> (i64, i64) {
        self.walk_vertical(1, x, y)
    }

    pub fn walk_north(&self, x: i64, y: i64) -> (i64, i64) {
        self.walk_vertical(-1, x, y)
    }
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut y = 0;
        for row in self.fields.iter() {
            let mut x = 0;
            for field in row.iter() {
                write!(f, "{}", field)?;
                x = x + 1;
            }
            write!(f, "\n",)?;
            y = y + 1;
        }
        Ok(())
    }
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Field::Tile => "#",
                Field::Empty => ".",
            }
        )
    }
}
