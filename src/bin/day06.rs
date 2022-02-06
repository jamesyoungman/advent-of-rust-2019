use std::collections::HashMap;
use std::collections::HashSet;
use std::io;
use std::io::BufRead;

fn string_to_oribit(s: &str) -> (String, String) {
    match s.split_once(')') {
        None => {
            panic!("should be a valid orbit");
        }
        Some((a, b)) => (a.to_owned(), b.to_owned()),
    }
}

fn build_tree(
    orbits: &[(String, String)],
) -> (
    Vec<String>,
    HashMap<String, String>,
    HashMap<String, Vec<String>>,
    HashSet<String>,
) {
    let mut all_bodies: HashSet<String> = HashSet::new();
    let mut parent_of: HashMap<String, String> = HashMap::new();
    let mut children_of: HashMap<String, Vec<String>> = HashMap::new();
    for (parent_name, child_name) in orbits {
        parent_of.insert(child_name.to_string(), parent_name.to_string());
        children_of
            .entry(parent_name.to_string())
            .or_insert_with(Vec::new)
            .push(child_name.to_string());
    }
    let mut non_orbiters: Vec<String> = Vec::new();
    for (parent, child) in orbits {
        if !parent_of.contains_key(parent) {
            non_orbiters.push(parent.to_string());
        }
        all_bodies.insert(parent.to_string());
        all_bodies.insert(child.to_string());
    }
    (non_orbiters, parent_of, children_of, all_bodies)
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

fn solve(orbits: &[(String, String)]) -> usize {
    let (_non_orbiters, parent_of, _children_of, all_bodies) = build_tree(orbits);
    count_orbits(&parent_of, &all_bodies)
}

#[test]
fn test_solve() {
    let test_input: Vec<&str> = vec![
        "COM)B", "B)C", "C)D", "D)E", "E)F", "B)G", "G)H", "D)I", "E)J", "J)K", "K)L",
    ];
    let orbits: Vec<(String, String)> = test_input.iter().cloned().map(string_to_oribit).collect();
    dbg!(&orbits);
    assert_eq!(solve(&orbits), 42);
}

fn part1(orbits: &[(String, String)]) {
    let count = solve(orbits);
    println!("Day 6 part 1: {} orbits", count);
}

fn main() {
    let orbits: Vec<(String, String)> = io::BufReader::new(io::stdin())
        .lines()
        .map(|s| string_to_oribit(s.unwrap().as_str()))
        .collect();
    part1(&orbits);
}
