use std::collections::{HashMap, HashSet};

use lib::error::Fail;
use lib::input::{read_file_as_lines, run_with_input};

fn build_tree(orbits: &[(String, String)]) -> (HashMap<String, String>, HashSet<String>) {
    let mut all_bodies: HashSet<String> = HashSet::new();
    let mut parent_of: HashMap<String, String> = HashMap::new();
    for (parent_name, child_name) in orbits {
        parent_of.insert(child_name.to_string(), parent_name.to_string());
    }
    for (parent, child) in orbits {
        all_bodies.insert(parent.to_string());
        all_bodies.insert(child.to_string());
    }
    (parent_of, all_bodies)
}

fn count_orbits(parent_of: &HashMap<String, String>, all_bodies: &HashSet<String>) -> usize {
    fn count_parents(name: &str, parent_of: &HashMap<String, String>) -> usize {
        match parent_of.get(&name.to_string()) {
            None => 0,
            Some(parent_name) => 1 + count_parents(parent_name, parent_of),
        }
    }
    all_bodies
        .iter()
        .map(|name| count_parents(name, parent_of))
        .sum()
}

#[test]
fn test_count_orbits() {
    let test_input: Vec<&str> = vec![
        "COM)B", "B)C", "C)D", "D)E", "E)F", "B)G", "G)H", "D)I", "E)J", "J)K", "K)L",
    ];
    let orbits: Vec<(String, String)> = test_input
        .iter()
        .cloned()
        .map(string_to_oribit)
        .map(|x| x.expect("test data should be valid"))
        .collect::<Vec<(String, String)>>();
    let (parent_of, all_bodies) = build_tree(&orbits);
    assert_eq!(count_orbits(&parent_of, &all_bodies), 42);
}

fn compute_transfer_counts(
    mut who: String,
    parent_of: &HashMap<String, String>,
) -> HashMap<String, usize> {
    let mut result: HashMap<String, usize> = HashMap::new();
    let mut count: usize = 0;
    loop {
        match parent_of.get(&who) {
            Some(p) => {
                result.insert(p.to_string(), count);
                count += 1;
                who = p.to_string();
            }
            None => {
                return result;
            }
        }
    }
}

fn count_transfers(from: String, to: String, parent_of: &HashMap<String, String>) -> Option<usize> {
    let transfers_to = compute_transfer_counts(from, parent_of);
    let mut body = to;
    let mut transfers: usize = 0;
    loop {
        match parent_of.get(&body) {
            None => {
                return None;
            }
            Some(p) => match transfers_to.get(p) {
                None => {
                    transfers += 1;
                    body = p.to_string();
                }
                Some(n) => {
                    return Some(n + transfers);
                }
            },
        }
    }
}

#[test]
fn test_count_transfers() {
    let test_input: Vec<&str> = vec![
        "COM)B", "B)C", "C)D", "D)E", "E)F", "B)G", "G)H", "D)I", "E)J", "J)K", "K)L", "K)YOU",
        "I)SAN",
    ];
    let orbits: Vec<(String, String)> = test_input
        .iter()
        .cloned()
        .map(|s| string_to_oribit(s))
        .map(|x| x.expect("test data should be valid"))
        .collect();

    let (parent_of, _all_bodies) = build_tree(&orbits);
    assert_eq!(
        count_transfers("YOU".to_string(), "SAN".to_string(), &parent_of),
        Some(4)
    );
}

fn part1(parent_of: &HashMap<String, String>, all_bodies: &HashSet<String>) {
    println!(
        "Day 6 part 1: {} orbits",
        count_orbits(parent_of, all_bodies)
    );
}

fn part2(parent_of: &HashMap<String, String>) {
    match count_transfers("YOU".to_string(), "SAN".to_string(), parent_of) {
        Some(n) => {
            println!("Day 6 part 2: {} transfers", n);
        }
        None => {
            println!("Day 6 part 2: no solution found");
        }
    }
}

fn string_to_oribit(s: &str) -> Result<(String, String), Fail> {
    if let Some((a, b)) = s.split_once(')') {
        Ok((a.to_owned(), b.to_owned()))
    } else {
        Err(Fail(format!(
            "'{}' should be a valid orbit but it is not",
            s
        )))
    }
}

fn run(input: Vec<String>) -> Result<(), Fail> {
    let parsed: Result<Vec<(String, String)>, Fail> = input
        .into_iter()
        .map(|s: String| -> Result<(String, String), Fail> { string_to_oribit(s.as_str()) })
        .collect();
    match parsed {
        Ok(orbits) => {
            let (parent_of, all_bodies) = build_tree(&orbits);
            part1(&parent_of, &all_bodies);
            part2(&parent_of);
            Ok(())
        }
        Err(e) => Err(e),
    }
}

fn main() -> Result<(), Fail> {
    run_with_input(6, read_file_as_lines, run)
}
