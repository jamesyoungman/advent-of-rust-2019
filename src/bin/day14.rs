use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};

use aoc::read_stdin_lines;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Chemical(String);

impl Chemical {
    fn new(s: &str) -> Chemical {
        Chemical(s.to_string())
    }

    fn is_ore(&self) -> bool {
        self.0.as_str() == "ORE"
    }
}

impl Display for Chemical {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.0.as_str())
    }
}

type Quantity = i64;

#[derive(Debug)]
struct Reagent {
    quantity: Quantity,
    chemical: Chemical,
}

impl Display for Reagent {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.quantity, self.chemical.0.as_str())
    }
}

#[derive(Debug)]
enum BadInput {
    FormatError(String),
}

impl Display for BadInput {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            BadInput::FormatError(msg) => {
                write!(f, "input format error: {}", msg)
            }
        }
    }
}

impl TryFrom<&str> for Reagent {
    type Error = BadInput;
    fn try_from(s: &str) -> Result<Reagent, BadInput> {
        match s.split_once(' ') {
            Some((q, c)) => match q.parse() {
                Ok(n) => Ok(Reagent {
                    quantity: n,
                    chemical: Chemical(c.to_string()),
                }),
                Err(e) => Err(BadInput::FormatError(format!(
                    "invalid number '{}': {}",
                    q, e
                ))),
            },
            None => Err(BadInput::FormatError(format!(
                "expected 'QTY CHEMICAL' pair, got {}",
                s
            ))),
        }
    }
}

#[derive(Debug)]
struct Recipe {
    inputs: Vec<Reagent>,
    output: Reagent,
}

impl Recipe {
    fn multiplier_to_produce(&self, quantity: &Quantity) -> i64 {
        let d = self.output.quantity;
        (quantity + d - 1) / d
    }
}

#[test]
fn test_multiplier_to_produce() {
    let r1 = Recipe {
        inputs: vec![Reagent {
            quantity: 9,
            chemical: Chemical("ORE".to_string()),
        }],
        output: Reagent {
            quantity: 2,
            chemical: Chemical("A".to_string()),
        },
    };
    assert_eq!(5, r1.multiplier_to_produce(&10));
    assert_eq!(6, r1.multiplier_to_produce(&11));
}

impl TryFrom<&str> for Recipe {
    type Error = BadInput;
    fn try_from(s: &str) -> Result<Recipe, BadInput> {
        match s.split_once(" => ") {
            Some((lhs, rhs)) => {
                fn string_list_to_reagents(s: &str) -> Result<Vec<Reagent>, BadInput> {
                    s.split(", ").map(Reagent::try_from).collect()
                }

                let inputs = string_list_to_reagents(lhs)?;
                let output = Reagent::try_from(rhs)?;
                Ok(Recipe { inputs, output })
            }
            None => Err(BadInput::FormatError(
                "expected recipe to contain ' => '".to_string(),
            )),
        }
    }
}

fn parse_recipes<S: AsRef<str>>(input: &[S]) -> Result<Vec<Recipe>, BadInput> {
    input.iter().map(|s| Recipe::try_from(s.as_ref())).collect()
}

fn make_recipe_map(recipes: Vec<Recipe>) -> HashMap<Chemical, Recipe> {
    let mut result = HashMap::new();
    for recipe in recipes.into_iter() {
        result.insert(recipe.output.chemical.to_owned(), recipe);
    }
    result.insert(
        Chemical::new("ORE"),
        Recipe {
            // You "make" ORE from nothing.
            inputs: Vec::with_capacity(0),
            output: Reagent {
                quantity: 1,
                chemical: Chemical::new("ORE"),
            },
        },
    );
    result
}

struct Wanted {
    items: Vec<(Chemical, Quantity)>,
}

impl Wanted {
    fn new() -> Wanted {
        Wanted { items: Vec::new() }
    }

    fn pop(&mut self) -> Option<(Chemical, Quantity)> {
        self.items.pop()
    }

    fn push(&mut self, item: (Chemical, Quantity)) {
        match self
            .items
            .iter_mut()
            .find(|(chemical, _)| chemical == &item.0)
            .map(|(_, qty)| qty)
        {
            Some(n) => {
                *n += item.1;
                return;
            }
            None => {
                self.items.push(item);
            }
        }
    }
}

fn ore_cost_of(
    wanted: &mut Wanted,
    stock: &mut HashMap<Chemical, Quantity>,
    mapping: &HashMap<Chemical, Recipe>,
) -> Result<Quantity, String> {
    let mut ore_used = 0;
    while let Some((make_chemical, need_quantity)) = wanted.pop() {
        let recipe = if let Some(recipe) = mapping.get(&make_chemical) {
            recipe
        } else {
            return Err(format!(
                "Need {} but there is no way to make it",
                &make_chemical
            ));
        };
        let multiplier = recipe.multiplier_to_produce(&need_quantity);
        let make_quantity = recipe.output.quantity * multiplier;
        assert!(make_quantity >= need_quantity);

        if !make_chemical.is_ore() {
            print!("Consume");
            for (i, input) in recipe.inputs.iter().enumerate() {
                print!(
                    "{} {} {}",
                    if i > 0 { "," } else { "" },
                    input.quantity * multiplier,
                    &input.chemical
                );
            }
            println!(" to produce {} {}", make_quantity, make_chemical);
        }

        for input in recipe.inputs.iter() {
            let needed = input.quantity * multiplier;
            assert!(needed >= 0);
            if input.chemical.is_ore() {
                // we never have ore "on hand" i.e. left over as a prodct
                // from a previous transformation.
                ore_used += needed;
            }
            let onhand = stock.entry(input.chemical.clone()).or_insert(0);
            assert!(*onhand >= 0);
            if *onhand >= needed {
                *onhand -= needed;
            } else {
                let deficit = needed - *onhand;
                assert!(deficit > 0);
                *onhand = 0;
                wanted.push((input.chemical.clone(), deficit));
            }
        }
        let left_over = make_quantity - need_quantity;
        assert!(left_over >= 0);
        *stock.entry(make_chemical.clone()).or_insert(0) += left_over;
    }
    Ok(ore_used)
}

fn solve1(mapping: &HashMap<Chemical, Recipe>) -> Result<Quantity, String> {
    let mut wanted = Wanted::new();
    wanted.push((Chemical::new("FUEL"), 1));
    let mut stock = HashMap::new();
    ore_cost_of(&mut wanted, &mut stock, mapping)
}

#[test]
fn test_solve1_example1() {
    let recipes: Vec<Recipe> = parse_recipes(&[
        "9 ORE => 2 A",
        "8 ORE => 3 B",
        "7 ORE => 5 C",
        "3 A, 4 B => 1 AB",
        "5 B, 7 C => 1 BC",
        "4 C, 1 A => 1 CA",
        "2 AB, 3 BC, 4 CA => 1 FUEL",
    ])
    .expect("example 1 should be valid");
    let mapping = make_recipe_map(recipes);
    assert_eq!(solve1(&mapping), Ok(165));
}

#[test]
fn test_solve1_example2() {
    let recipes: Vec<Recipe> = parse_recipes(&[
        "157 ORE => 5 NZVS",
        "165 ORE => 6 DCFZ",
        "44 XJWVT, 5 KHKGT, 1 QDVJ, 29 NZVS, 9 GPVTF, 48 HKGWZ => 1 FUEL",
        "12 HKGWZ, 1 GPVTF, 8 PSHF => 9 QDVJ",
        "179 ORE => 7 PSHF",
        "177 ORE => 5 HKGWZ",
        "7 DCFZ, 7 PSHF => 2 XJWVT",
        "165 ORE => 2 GPVTF",
        "3 DCFZ, 7 NZVS, 5 HKGWZ, 10 PSHF => 8 KHKGT",
    ])
    .expect("part 1 example 2 should be valid");
    let mapping = make_recipe_map(recipes);
    assert_eq!(solve1(&mapping), Ok(13312));
}

#[test]
fn test_solve1_example3() {
    let recipes: Vec<Recipe> = parse_recipes(&[
        "2 VPVL, 7 FWMGM, 2 CXFTF, 11 MNCFX => 1 STKFG",
        "17 NVRVD, 3 JNWZP => 8 VPVL",
        "53 STKFG, 6 MNCFX, 46 VJHF, 81 HVMC, 68 CXFTF, 25 GNMV => 1 FUEL",
        "22 VJHF, 37 MNCFX => 5 FWMGM",
        "139 ORE => 4 NVRVD",
        "144 ORE => 7 JNWZP",
        "5 MNCFX, 7 RFSQX, 2 FWMGM, 2 VPVL, 19 CXFTF => 3 HVMC",
        "5 VJHF, 7 MNCFX, 9 VPVL, 37 CXFTF => 6 GNMV",
        "145 ORE => 6 MNCFX",
        "1 NVRVD => 8 CXFTF",
        "1 VJHF, 6 MNCFX => 4 RFSQX",
        "176 ORE => 6 VJHF",
    ])
    .expect("part 1 example 3 should be valid");
    let mapping = make_recipe_map(recipes);
    assert_eq!(solve1(&mapping), Ok(180697));
}

#[test]
fn test_solve1_example4() {
    let recipes: Vec<Recipe> = parse_recipes(&[
        "171 ORE => 8 CNZTR",
        "7 ZLQW, 3 BMBT, 9 XCVML, 26 XMNCP, 1 WPTQ, 2 MZWV, 1 RJRHP => 4 PLWSL",
        "114 ORE => 4 BHXH",
        "14 VRPVC => 6 BMBT",
        "6 BHXH, 18 KTJDG, 12 WPTQ, 7 PLWSL, 31 FHTLT, 37 ZDVW => 1 FUEL",
        "6 WPTQ, 2 BMBT, 8 ZLQW, 18 KTJDG, 1 XMNCP, 6 MZWV, 1 RJRHP => 6 FHTLT",
        "15 XDBXC, 2 LTCX, 1 VRPVC => 6 ZLQW",
        "13 WPTQ, 10 LTCX, 3 RJRHP, 14 XMNCP, 2 MZWV, 1 ZLQW => 1 ZDVW",
        "5 BMBT => 4 WPTQ",
        "189 ORE => 9 KTJDG",
        "1 MZWV, 17 XDBXC, 3 XCVML => 2 XMNCP",
        "12 VRPVC, 27 CNZTR => 2 XDBXC",
        "15 KTJDG, 12 BHXH => 5 XCVML",
        "3 BHXH, 2 VRPVC => 7 MZWV",
        "121 ORE => 7 VRPVC",
        "7 XCVML => 6 RJRHP",
        "5 BHXH, 4 VRPVC => 5 LTCX",
    ])
    .expect("part 1 example 4 should be valid");
    let mapping = make_recipe_map(recipes);
    assert_eq!(solve1(&mapping), Ok(2210736));
}

fn part1(mapping: &HashMap<Chemical, Recipe>) {
    match solve1(mapping) {
        Ok(n) => {
            println!("Day 14 part 1: {}", n);
        }
        Err(e) => {
            eprintln!("Day 14 part 1: failed: {}", e);
        }
    }
}

fn main() {
    let lines = read_stdin_lines().expect("should be able to read input");
    match parse_recipes(&lines) {
        Ok(recipes) => {
            let mapping = make_recipe_map(recipes);
            part1(&mapping);
        }
        Err(e) => {
            eprintln!("invalid input: {}", e);
        }
    }
}
