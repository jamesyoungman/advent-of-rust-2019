use std::io;
use std::io::prelude::*;

fn fuel(mass: i64) -> i64 {
    mass / 3 - 2
}

fn cumulative_fuel(mass: i64) -> i64 {
    let mut tot: i64 = 0;
    let mut f = fuel(mass);
    while f > 0 {
        tot += f;
        f = fuel(f);
    }
    tot
}

#[test]
fn test_fuel() {
    assert!(fuel(12) == 2);
    assert!(fuel(14) == 2);
    assert!(fuel(1969) == 654);
    assert!(fuel(100756) == 33583);
}

fn main() {
    let masses: Vec<i64> = io::BufReader::new(io::stdin())
        .lines()
        .map(|s| s.unwrap().parse::<i64>().unwrap())
        .collect();
    let fuel1: i64 = masses.iter().map(|m| fuel(*m)).sum();
    println!("Day 01 part 1: fuel needed: {}", fuel1);
    let fuel2: i64 = masses.iter().map(|m: &i64| cumulative_fuel(*m)).sum();
    println!("Day 01 part 2: fuel needed: {}", fuel2);
}
