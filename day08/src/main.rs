#[macro_use]
extern crate log;
extern crate env_logger;

use std::convert::TryInto;
use std::fs::File;
use std::io::Read;

use std::boxed::Box;

use std::vec::Vec;

type Image = Vec<Layer>;
type Layer = Vec<Vec<u8>>;

fn parse_image(data: &str, width: usize, height: usize) -> Image {
  let mut image: Image = Vec::new();
  let mut layer: Layer = Vec::new();
  let mut row = Vec::new();
  let mut i = 1;
  let dim = width * height;
  for elem in data.chars() {
    if let Some(number) = elem.to_digit(10) {
      debug!("i={}: Adding {} to row {:?}", i, number, row);
      row.push(number.try_into().unwrap());
      if i % dim == 0 {
        layer.push(row);
        debug!("Layer complete, adding to image. layer={:?}", layer);
        image.push(layer);
        layer = Vec::new();
        row = Vec::new();
      } else if i % width == 0 {
        debug!("Row {:?} complete, adding to layer {:?} and starting new row", row, layer);
        layer.push(row);
        row = Vec::new();
      }
      i += 1;
    }
  }
  image
}

#[test]
fn test_parse_image() {
  let image = parse_image(&"123456789012"[..], 3, 2);
  assert_eq!(image.len(), 2);
  assert_eq!(image, vec![vec![vec![1, 2, 3], vec![4, 5, 6]], vec![vec![7, 8, 9], vec![0, 1, 2]]]);
}

fn count_occurences(layer: &Layer, digit: u8) -> usize {
  let mut count = 0;
  for row in layer.iter() {
    for elem in row.iter() {
      if *elem == digit {
        count = count + 1;
      }
    }
  }
  count
}

#[derive(Debug, PartialEq)]
enum Color {
  BLACK,
  WHITE,
  TRANSPARENT,
}

fn render_image(src: &Image, width: usize, height: usize) -> Vec<Vec<Color>> {
  let mut result = Vec::with_capacity(height);

  for i in 0..height {
    let mut row = Vec::with_capacity(width);

    for j in 0..width {
      let mut color = Color::TRANSPARENT;

      'outer: for layer in src.iter() {
        match layer[i][j] {
          0 => {
            color = Color::BLACK;
            break 'outer;
          }
          1 => {
            color = Color::WHITE;
            break 'outer;
          }
          2 => {
            // ignoring transparent
          }
          _ => panic!("Unsupported color"),
        }
      }

      row.push(color);
    }

    result.push(row);
  }
  result
}

fn main() {
  env_logger::init();

  let mut file = File::open("input.txt").unwrap();
  let mut contents = String::new();
  file.read_to_string(&mut contents).unwrap();

  let width = 25;
  let height = 6;

  let image = parse_image(&contents, width, height);
  println!("Number of layers: {}", image.len());

  let mut min_count = std::usize::MAX;
  let mut min_layer = None;
  for layer in image.iter() {
    let count = count_occurences(layer, 0);
    if count < min_count {
      min_count = count;
      min_layer = Some(Box::new(layer));
    }
  }

  if let Some(l) = min_layer {
    let ones = count_occurences(&l, 1);
    let twos = count_occurences(&l, 2);
    let result = ones * twos;
    println!("Part 1: {}", result);
  }

  let final_image = render_image(&image, width, height);
  let mut result = String::new();
  for row in final_image.iter() {
    for color in row.iter() {
      match color {
        Color::BLACK => result.push_str(&" "[..]),
        Color::WHITE => result.push_str(&"1"[..]),
        Color::TRANSPARENT => result.push_str(&" "[..]),
      }
    }
    result.push_str(&"\n"[..]);
  }
  println!("{}", result);
}
