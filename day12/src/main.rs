#[macro_use]
extern crate log;

extern crate num;

use std::cmp::Ordering;
use std::convert::TryInto;

use num::Integer;

type Position = (i64, i64, i64);
type Velocity = (i64, i64, i64);

#[derive(Debug, Copy, Clone, PartialEq)]
struct Moon {
  pos: Position,
  vel: Velocity,
}

impl Moon {
  pub fn new(x: i64, y: i64, z: i64) -> Self {
    Self {
      pos: (x, y, z),
      vel: (0, 0, 0),
    }
  }

  fn potential_energy(&self) -> u64 {
    let a = &self.pos;
    (a.0.abs() + a.1.abs() + a.2.abs()).try_into().unwrap()
  }

  fn kinetic_energy(&self) -> u64 {
    let a = &self.vel;
    (a.0.abs() + a.1.abs() + a.2.abs()).try_into().unwrap()
  }

  fn total_energy(&self) -> u64 {
    self.potential_energy() * self.kinetic_energy()
  }
}

struct Universe {
  moons: Vec<Moon>,
}

impl Universe {
  fn compute_gravity(x1: i64, x2: i64) -> i64 {
    match x1.cmp(&x2) {
      Ordering::Less => 1,
      Ordering::Equal => 0,
      Ordering::Greater => -1,
    }
  }

  pub fn apply_gravity(&mut self) {
    /* To apply gravity, consider every pair of moons. On each axis (x, y, and
     * z), the velocity of each moon changes by exactly +1 or -1 to pull the
     * moons together.
     */
    let n = self.moons.len();
    for i in 0..n {
      for j in i + 1..n {
        let m1 = &self.moons[i];
        let m2 = &self.moons[j];
        let g1 = Self::compute_gravity(m1.pos.0, m2.pos.0);
        let g2 = Self::compute_gravity(m1.pos.1, m2.pos.1);
        let g3 = Self::compute_gravity(m1.pos.2, m2.pos.2);
        let vel1 = (m1.vel.0 + g1, m1.vel.1 + g2, m1.vel.2 + g3);
        let vel2 = (m2.vel.0 - g1, m2.vel.1 - g2, m2.vel.2 - g3);
        self.moons[i].vel = vel1;
        self.moons[j].vel = vel2;
      }
    }
  }

  pub fn apply_velocity(&mut self) {
    for m in self.moons.iter_mut() {
      let pos = m.pos;
      let vel = m.vel;
      m.pos = (pos.0 + vel.0, pos.1 + vel.1, pos.2 + vel.2);
    }
  }

  pub fn progress_time(&mut self) {
    self.apply_gravity();
    self.apply_velocity();
  }

  pub fn total_energy(&self) -> u64 {
    self.moons.iter().map(|m| m.total_energy()).sum()
  }
}

fn part_one(moons: Vec<Moon>) {
  let mut uni = Universe { moons: moons };
  for _ in 0..1000 {
    uni.progress_time();
  }
  let total_energy = uni.total_energy();
  println!("Part one: {}", total_energy);
}

fn cycle_length(moons: Vec<Moon>) -> u64 {
  let initial = moons.clone();
  let n = initial.len();

  let mut uni = Universe { moons: moons };
  let mut x_period = None;
  let mut y_period = None;
  let mut z_period = None;
  let mut period: u64 = 0;
  while x_period.is_none() || y_period.is_none() || z_period.is_none() {
    uni.progress_time();
    period += 1;

    let mut x_period_found = true;
    let mut y_period_found = true;
    let mut z_period_found = true;
    for i in 0..n {
      if initial[i].pos.0 != uni.moons[i].pos.0 || initial[i].vel.0 != uni.moons[i].vel.0 {
        x_period_found = false;
      }
      if initial[i].pos.1 != uni.moons[i].pos.1 || initial[i].vel.1 != uni.moons[i].vel.1 {
        y_period_found = false;
      }
      if initial[i].pos.2 != uni.moons[i].pos.2 || initial[i].vel.2 != uni.moons[i].vel.2 {
        z_period_found = false;
      }
    }
    if x_period.is_none() && x_period_found {
      x_period = Some(period);
    }
    if y_period.is_none() && y_period_found {
      y_period = Some(period);
    }
    if z_period.is_none() && z_period_found {
      z_period = Some(period);
    }
  }
  let x = x_period.unwrap();
  let y = y_period.unwrap();
  let z = z_period.unwrap();
  debug!("Periods: x={}, y={}, z={}", x, y, z);
  let lcm = (x.lcm(&y)).lcm(&z);
  return lcm;
}

#[cfg(test)]
mod tests {
  use super::*;

  fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
  }

  #[test]
  fn test_motion_and_energy() {
    init();

    let moons = vec![Moon::new(-1, 0, 2), Moon::new(2, -10, -7), Moon::new(4, -8, 8), Moon::new(3, 5, -1)];
    let mut uni = Universe { moons: moons };

    uni.progress_time();

    // after 1 step
    assert_eq!(uni.moons[0].pos, (2, -1, 1));
    assert_eq!(uni.moons[0].vel, (3, -1, -1));

    assert_eq!(uni.moons[1].pos, (3, -7, -4));
    assert_eq!(uni.moons[1].vel, (1, 3, 3));

    assert_eq!(uni.moons[2].pos, (1, -7, 5));
    assert_eq!(uni.moons[2].vel, (-3, 1, -3));

    assert_eq!(uni.moons[3].pos, (2, 2, 0));
    assert_eq!(uni.moons[3].vel, (-1, -3, 1));

    uni.progress_time();

    // after 2 steps
    assert_eq!(uni.moons[0].pos, (5, -3, -1));
    assert_eq!(uni.moons[0].vel, (3, -2, -2));

    assert_eq!(uni.moons[1].pos, (1, -2, 2));
    assert_eq!(uni.moons[1].vel, (-2, 5, 6));

    assert_eq!(uni.moons[2].pos, (1, -4, -1));
    assert_eq!(uni.moons[2].vel, (0, 3, -6));

    assert_eq!(uni.moons[3].pos, (1, -4, 2));
    assert_eq!(uni.moons[3].vel, (-1, -6, 2));

    for _ in 2..10 {
      uni.progress_time();
    }

    // after 10 steps
    assert_eq!(uni.moons[0].pos, (2, 1, -3));
    assert_eq!(uni.moons[0].vel, (-3, -2, 1));

    assert_eq!(uni.moons[1].pos, (1, -8, 0));
    assert_eq!(uni.moons[1].vel, (-1, 1, 3));

    assert_eq!(uni.moons[2].pos, (3, -6, 1));
    assert_eq!(uni.moons[2].vel, (3, 2, -3));

    assert_eq!(uni.moons[3].pos, (2, 0, 4));
    assert_eq!(uni.moons[3].vel, (1, -1, -1));

    assert_eq!(uni.total_energy(), 179);
  }

  #[test]
  fn test_cycle_length() {
    init();

    assert_eq!(
      cycle_length(vec![
        Moon::new(-1, 0, 2),
        Moon::new(2, -10, -7),
        Moon::new(4, -8, 8),
        Moon::new(3, 5, -1)
      ]),
      2772
    );

    assert_eq!(
      cycle_length(vec![
        Moon::new(-8, -10, 0),
        Moon::new(5, 5, 10),
        Moon::new(2, -7, 3),
        Moon::new(9, -8, -3)
      ]),
      4686774924
    );
  }
}

fn part_two(moons: Vec<Moon>) {
  let n = cycle_length(moons);
  println!("Part two: {}", n);
}

fn main() {
  env_logger::init();

  let moons = vec![
    Moon::new(1, 4, 4),
    Moon::new(-4, -1, 19),
    Moon::new(-15, -14, 12),
    Moon::new(-17, 1, 10),
  ];

  part_one(moons.clone());
  part_two(moons.clone());
}
