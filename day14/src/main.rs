#[macro_use]
extern crate log;
extern crate env_logger;

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

#[derive(Debug, PartialEq, Hash, Eq)]
struct Ingredient<'a> {
  unit: &'a str,
  amount: u64,
}

impl<'a> Ingredient<'a> {
  pub fn parse(s: &str) -> Result<Ingredient, String> {
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() != 2 {
      return Err(format!("Expected two parts, but got {:?}", parts));
    }
    let amount = parts[0].parse().map_err(|e| format!("Unable to parse amount: {:?}", e))?;
    let unit = parts[1];
    return Ok(Ingredient {
      unit: unit,
      amount: amount,
    });
  }
}

#[derive(Debug, PartialEq, Eq)]
struct Formula<'a> {
  input: Vec<Ingredient<'a>>,
  output: Ingredient<'a>,
}

impl<'a> Formula<'a> {
  pub fn produce_output(&self, amount: u64) -> (Vec<Ingredient>, u64) {
    let m = self.output.amount;
    let rem = amount % self.output.amount;
    debug_assert_eq!(rem < m, true);
    let leftover = if rem > 0 { m - rem } else { 0 };
    let amount_adjusted = if rem > 0 { amount + leftover } else { amount };

    let factor = amount_adjusted / m;
    let mut ingredients = Vec::new();
    for ing in self.input.iter() {
      let ing_amount_needed = ing.amount * factor;
      ingredients.push(Ingredient {
        unit: ing.unit,
        amount: ing_amount_needed,
      })
    }
    return (ingredients, leftover);
  }

  pub fn only_need_ore(&self) -> bool {
    self.input.len() == 1 && self.input[0].unit == &"ORE"[..]
  }

  pub fn parse(s: &str) -> Result<Formula, String> {
    let parts: Vec<&str> = s.split("=>").collect();
    if parts.len() != 2 {
      return Err(format!("Split error at =>: {:?}", parts));
    }

    let lhs = parts[0];
    let mut input = Vec::new();
    for token in lhs.split(',') {
      let ingredient = Ingredient::parse(token)?;
      input.push(ingredient);
    }

    let rhs = parts[1];
    let output = Ingredient::parse(rhs)?;

    return Ok(Formula {
      input: input,
      output: output,
    });
  }
}

fn parse_relations(s: &str) -> Result<Vec<Formula>, String> {
  let mut result = Vec::new();
  for line in s.split('\n') {
    let line = line.trim();
    if line.is_empty() {
      debug!("Skipping empty line");
      continue;
    }
    debug!("Parsing line: {}", line);
    let reaction = match Formula::parse(line) {
      Ok(a) => a,
      Err(e) => return Err(format!("Unable to parse line: {:?}. Error: {:?}", line, e)),
    };
    result.push(reaction);
  }
  return Ok(result);
}

fn minimum_ore_for_fuel(relations: &Vec<Formula>, fuel_amount_desired: u64) -> u64 {
  let mut formulas = HashMap::new();
  for i in 0..relations.len() {
    let r = &relations[i];
    formulas.insert(r.output.unit, r);
  }

  let mut rdeps: HashMap<&str, u64> = HashMap::new();
  let mut leftovers: HashMap<&str, u64> = HashMap::new();
  rdeps.insert(&"FUEL"[..], fuel_amount_desired);

  let mut update_found = true;
  while update_found {
    update_found = false;
    let mut new_rdeps: HashMap<&str, u64> = HashMap::new();

    for (unit, amount) in rdeps.iter() {
      // we need 'amount' of 'unit'
      let formula = &formulas.get(unit).expect("Formula not found");

      if formula.only_need_ore() {
        debug!("Not resolving dependency {} {} because it only depends on ORE", amount, unit);
        let entry = new_rdeps.entry(unit).or_insert(0);
        *entry = *entry + *amount;
      } else {
        update_found = true;
        let mut amount_needed = *amount;
        let leftover = *leftovers.get(unit).unwrap_or(&0);
        if leftover > 0 {
          debug!("Leftover available: {} {}", leftover, unit);
          let usable_leftover = if leftover >= amount_needed { amount_needed } else { leftover };
          debug!("Usable amount: {} {}", usable_leftover, unit);
          amount_needed -= usable_leftover;
          leftovers.insert(unit, leftover - usable_leftover);
        }

        debug!("Replacing {} {} using its formula", amount_needed, unit);
        let (ingredients, leftover) = formula.produce_output(amount_needed);
        if leftover > 0 {
          debug!("Adding to leftovers: {} {}", leftover, unit);
          let entry = leftovers.entry(unit).or_insert(0);
          *entry = *entry + leftover;
        }

        debug!("Adding ingredients to new_rdeps");
        for ingredient in ingredients {
          let entry = new_rdeps.entry(ingredient.unit).or_insert(0);
          debug!("Adding {} {}", ingredient.amount, ingredient.unit);
          *entry = *entry + ingredient.amount;
        }
      }
    }
    rdeps = new_rdeps;
    debug!("rdeps: {:?}", rdeps);
    debug!("leftovers: {:?}", leftovers);
  }

  debug!("rdeps: {:?}", rdeps);
  debug!("leftovers: {:?}", leftovers);
  let mut total_ore = 0;
  for (unit, amount) in rdeps.iter() {
    let formula = formulas.get(unit).expect("formula must exist");
    let (ingredients, _) = formula.produce_output(*amount);
    debug_assert_eq!(ingredients.len(), 1);
    debug_assert_eq!(ingredients[0].unit, &"ORE"[..]);
    let ore_amount = ingredients[0].amount;
    debug!("Need {} {} => produced by {} ORE", amount, unit, ore_amount);
    total_ore += ore_amount;
  }
  return total_ore;
}

fn ore_to_fuel(relations: &Vec<Formula>, ore_available: u64) -> u64 {
  let mut lower = ore_available / minimum_ore_for_fuel(relations, 1);
  let mut upper = 2 * lower; // this is an uneducated guess, let's see if it works
  debug!("Guessing initial bounds: lower={}, upper={}", lower, upper);

  while lower < upper {
    let mid = (lower + upper + 1) / 2;
    let ore = minimum_ore_for_fuel(relations, mid);
    if ore > ore_available {
      debug!("solution must be in lower half");
      upper = mid - 1;
    } else {
      debug!("solution must be in upper half");
      lower = mid;
    }
    debug!("New bounds: lower={}, upper={}", lower, upper);
  }
  debug_assert_eq!(lower, upper);
  return upper;
}

#[cfg(test)]
mod tests {
  use super::*;

  fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
  }

  #[test]
  fn test_parse_ingredient() {
    init();

    assert_eq!(
      Ingredient::parse(&"10 AB"[..]),
      Ok(Ingredient {
        unit: &"AB"[..],
        amount: 10
      })
    );
    assert_eq!(Ingredient::parse(&"1 A"[..]), Ok(Ingredient { unit: &"A"[..], amount: 1 }));
  }

  #[test]
  fn test_parse_relations() {
    init();

    let s = "2 KBRD => 3 NSPQ
1 TMTNM, 5 WMZD => 4 JVBK";
    let relations = parse_relations(&s[..]).unwrap();
    assert_eq!(
      relations,
      vec![
        Formula {
          input: vec![Ingredient {
            amount: 2,
            unit: &"KBRD"[..]
          }],
          output: Ingredient {
            amount: 3,
            unit: &"NSPQ"[..]
          }
        },
        Formula {
          input: vec![
            Ingredient {
              amount: 1,
              unit: &"TMTNM"[..]
            },
            Ingredient {
              amount: 5,
              unit: &"WMZD"[..]
            }
          ],
          output: Ingredient {
            amount: 4,
            unit: &"JVBK"[..],
          }
        }
      ]
    );
  }

  #[test]
  fn test_minimum_ore_for_fuel_small() {
    init();

    let s = "9 ORE => 2 A
8 ORE => 3 B
7 ORE => 5 C
3 A, 4 B => 1 AB
5 B, 7 C => 1 BC
4 C, 1 A => 1 CA
2 AB, 3 BC, 4 CA => 1 FUEL";

    let relations = parse_relations(&s[..]).unwrap();
    let ore = minimum_ore_for_fuel(&relations, 1);
    assert_eq!(ore, 165);
  }

  #[test]
  fn test_minimum_ore_for_fuel_large1() {
    init();

    let s = "157 ORE => 5 NZVS
165 ORE => 6 DCFZ
44 XJWVT, 5 KHKGT, 1 QDVJ, 29 NZVS, 9 GPVTF, 48 HKGWZ => 1 FUEL
12 HKGWZ, 1 GPVTF, 8 PSHF => 9 QDVJ
179 ORE => 7 PSHF
177 ORE => 5 HKGWZ
7 DCFZ, 7 PSHF => 2 XJWVT
165 ORE => 2 GPVTF
3 DCFZ, 7 NZVS, 5 HKGWZ, 10 PSHF => 8 KHKGT";

    let relations = parse_relations(&s[..]).unwrap();
    let ore = minimum_ore_for_fuel(&relations, 1);
    assert_eq!(ore, 13312);
  }

  #[test]
  fn test_minimum_ore_for_fuel_large2() {
    init();

    let s = "2 VPVL, 7 FWMGM, 2 CXFTF, 11 MNCFX => 1 STKFG
17 NVRVD, 3 JNWZP => 8 VPVL
53 STKFG, 6 MNCFX, 46 VJHF, 81 HVMC, 68 CXFTF, 25 GNMV => 1 FUEL
22 VJHF, 37 MNCFX => 5 FWMGM
139 ORE => 4 NVRVD
144 ORE => 7 JNWZP
5 MNCFX, 7 RFSQX, 2 FWMGM, 2 VPVL, 19 CXFTF => 3 HVMC
5 VJHF, 7 MNCFX, 9 VPVL, 37 CXFTF => 6 GNMV
145 ORE => 6 MNCFX
1 NVRVD => 8 CXFTF
1 VJHF, 6 MNCFX => 4 RFSQX
176 ORE => 6 VJHF";

    let relations = parse_relations(&s[..]).unwrap();
    let ore = minimum_ore_for_fuel(&relations, 1);
    assert_eq!(ore, 180697);
  }

  #[test]
  fn test_minimum_ore_for_fuel_large3() {
    init();

    let s = "171 ORE => 8 CNZTR
7 ZLQW, 3 BMBT, 9 XCVML, 26 XMNCP, 1 WPTQ, 2 MZWV, 1 RJRHP => 4 PLWSL
114 ORE => 4 BHXH
14 VRPVC => 6 BMBT
6 BHXH, 18 KTJDG, 12 WPTQ, 7 PLWSL, 31 FHTLT, 37 ZDVW => 1 FUEL
6 WPTQ, 2 BMBT, 8 ZLQW, 18 KTJDG, 1 XMNCP, 6 MZWV, 1 RJRHP => 6 FHTLT
15 XDBXC, 2 LTCX, 1 VRPVC => 6 ZLQW
13 WPTQ, 10 LTCX, 3 RJRHP, 14 XMNCP, 2 MZWV, 1 ZLQW => 1 ZDVW
5 BMBT => 4 WPTQ
189 ORE => 9 KTJDG
1 MZWV, 17 XDBXC, 3 XCVML => 2 XMNCP
12 VRPVC, 27 CNZTR => 2 XDBXC
15 KTJDG, 12 BHXH => 5 XCVML
3 BHXH, 2 VRPVC => 7 MZWV
121 ORE => 7 VRPVC
7 XCVML => 6 RJRHP
5 BHXH, 4 VRPVC => 5 LTCX";

    let relations = parse_relations(&s[..]).unwrap();
    let ore = minimum_ore_for_fuel(&relations, 1);
    assert_eq!(ore, 2210736);
  }

  #[test]
  fn test_produce_output() {
    init();

    let formula = Formula::parse(&"12 HKGWZ, 1 GPVTF, 8 PSHF => 9 QDVJ"[..]).unwrap();
    let (ingredients, leftover) = formula.produce_output(1);
    assert_eq!(leftover, 8);
    assert_eq!(
      ingredients,
      vec![
        Ingredient {
          unit: &"HKGWZ"[..],
          amount: 12
        },
        Ingredient {
          unit: &"GPVTF"[..],
          amount: 1
        },
        Ingredient {
          unit: &"PSHF"[..],
          amount: 8
        }
      ]
    );

    let (ingredients, leftover) = formula.produce_output(10);
    assert_eq!(leftover, 8);
    assert_eq!(
      ingredients,
      vec![
        Ingredient {
          unit: &"HKGWZ"[..],
          amount: 24
        },
        Ingredient {
          unit: &"GPVTF"[..],
          amount: 2
        },
        Ingredient {
          unit: &"PSHF"[..],
          amount: 16
        }
      ]
    );
  }

  #[test]
  fn test_produce_output2() {
    init();

    let formula = Formula::parse(&"44 XJWVT, 5 KHKGT => 1 FUEL"[..]).unwrap();
    let (_ingredients, leftover) = formula.produce_output(1);
    assert_eq!(leftover, 0);
  }

  #[test]
  fn test_ore_to_fuel1() {
    init();

    let s = "157 ORE => 5 NZVS
165 ORE => 6 DCFZ
44 XJWVT, 5 KHKGT, 1 QDVJ, 29 NZVS, 9 GPVTF, 48 HKGWZ => 1 FUEL
12 HKGWZ, 1 GPVTF, 8 PSHF => 9 QDVJ
179 ORE => 7 PSHF
177 ORE => 5 HKGWZ
7 DCFZ, 7 PSHF => 2 XJWVT
165 ORE => 2 GPVTF
3 DCFZ, 7 NZVS, 5 HKGWZ, 10 PSHF => 8 KHKGT";

    let relations = parse_relations(&s[..]).unwrap();
    let fuel = ore_to_fuel(&relations, 1000000000000);
    assert_eq!(fuel, 82892753);
  }

  #[test]
  fn test_ore_to_fuel2() {
    init();

    let s = "2 VPVL, 7 FWMGM, 2 CXFTF, 11 MNCFX => 1 STKFG
17 NVRVD, 3 JNWZP => 8 VPVL
53 STKFG, 6 MNCFX, 46 VJHF, 81 HVMC, 68 CXFTF, 25 GNMV => 1 FUEL
22 VJHF, 37 MNCFX => 5 FWMGM
139 ORE => 4 NVRVD
144 ORE => 7 JNWZP
5 MNCFX, 7 RFSQX, 2 FWMGM, 2 VPVL, 19 CXFTF => 3 HVMC
5 VJHF, 7 MNCFX, 9 VPVL, 37 CXFTF => 6 GNMV
145 ORE => 6 MNCFX
1 NVRVD => 8 CXFTF
1 VJHF, 6 MNCFX => 4 RFSQX
176 ORE => 6 VJHF";

    let relations = parse_relations(&s[..]).unwrap();
    let fuel = ore_to_fuel(&relations, 1000000000000);
    assert_eq!(fuel, 5586022);
  }

  #[test]
  fn test_ore_to_fuel3() {
    init();

    let s = "171 ORE => 8 CNZTR
7 ZLQW, 3 BMBT, 9 XCVML, 26 XMNCP, 1 WPTQ, 2 MZWV, 1 RJRHP => 4 PLWSL
114 ORE => 4 BHXH
14 VRPVC => 6 BMBT
6 BHXH, 18 KTJDG, 12 WPTQ, 7 PLWSL, 31 FHTLT, 37 ZDVW => 1 FUEL
6 WPTQ, 2 BMBT, 8 ZLQW, 18 KTJDG, 1 XMNCP, 6 MZWV, 1 RJRHP => 6 FHTLT
15 XDBXC, 2 LTCX, 1 VRPVC => 6 ZLQW
13 WPTQ, 10 LTCX, 3 RJRHP, 14 XMNCP, 2 MZWV, 1 ZLQW => 1 ZDVW
5 BMBT => 4 WPTQ
189 ORE => 9 KTJDG
1 MZWV, 17 XDBXC, 3 XCVML => 2 XMNCP
12 VRPVC, 27 CNZTR => 2 XDBXC
15 KTJDG, 12 BHXH => 5 XCVML
3 BHXH, 2 VRPVC => 7 MZWV
121 ORE => 7 VRPVC
7 XCVML => 6 RJRHP
5 BHXH, 4 VRPVC => 5 LTCX";

    let relations = parse_relations(&s[..]).unwrap();
    let fuel = ore_to_fuel(&relations, 1000000000000);
    assert_eq!(fuel, 460664);
  }
}

fn main() -> std::io::Result<()> {
  env_logger::init();
  let mut file = File::open("input.txt")?;
  let mut contents = String::new();
  file.read_to_string(&mut contents)?;

  let relations = parse_relations(&contents).expect("Unable to parse input file");
  let total_ore = minimum_ore_for_fuel(&relations, 1);
  println!("Part One: {}", total_ore);

  let fuel = ore_to_fuel(&relations, 1000000000000);
  println!("Part Two: {}", fuel);

  Ok(())
}
