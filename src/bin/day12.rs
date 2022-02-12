use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

use regex::Regex;

use aoc::read_stdin_lines;

const DIMENSIONS: usize = 3;

#[derive(Debug)]
struct SimulationFlags {
    verbose: bool,
}

#[derive(Debug)]
struct Overflow {}

impl Display for Overflow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("arithmetic overflow")
    }
}

impl std::error::Error for Overflow {}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Clone, Copy)]
struct Distance(i32);
#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Clone, Copy)]
struct Velocity(i32);

impl Display for Distance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl Display for Velocity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl Distance {
    fn add(&self, other: Velocity) -> Result<Distance, Overflow> {
        match self.0.checked_add(other.0) {
            Some(n) => Ok(Distance(n)),
            None => Err(Overflow {}),
        }
    }
    fn sub(&self, other: Velocity) -> Result<Distance, Overflow> {
        match self.0.checked_sub(other.0) {
            Some(n) => Ok(Distance(n)),
            None => Err(Overflow {}),
        }
    }
}

impl Velocity {
    fn add(&self, other: Velocity) -> Result<Velocity, Overflow> {
        match self.0.checked_add(other.0) {
            Some(n) => Ok(Velocity(n)),
            None => Err(Overflow {}),
        }
    }
    fn sub(&self, other: Velocity) -> Result<Velocity, Overflow> {
        match self.0.checked_sub(other.0) {
            Some(n) => Ok(Velocity(n)),
            None => Err(Overflow {}),
        }
    }
}

struct IntegerExtractor {
    re: Regex,
}

impl IntegerExtractor {
    pub fn new() -> IntegerExtractor {
        IntegerExtractor {
            re: Regex::new(r"[+-]?\d+").unwrap(),
        }
    }

    pub fn get_integers<T, S>(&self, s: S) -> Result<Vec<T>, <T as FromStr>::Err>
    where
        S: AsRef<str>,
        T: FromStr + Debug,
    {
        self.re
            .captures_iter(s.as_ref())
            .map(|cap| cap[0].parse::<T>())
            .collect()
    }
}

#[derive(Debug)]
struct BadInput(String);

impl Display for BadInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "bad input: {}", &self.0)
    }
}

impl std::error::Error for BadInput {}

struct System1D {
    position: Vec<Distance>,
    velocity: Vec<Velocity>,
    size: usize,
}

impl System1D {
    fn body_count(&self) -> usize {
        self.position.len()
    }

    fn new(positions: &[Distance], velocities: &[Velocity]) -> System1D {
        assert_eq!(positions.len(), velocities.len());
        System1D {
            position: positions.iter().copied().collect(),
            velocity: velocities.iter().copied().collect(),
            size: positions.len(),
        }
    }

    fn step(&mut self, _: &SimulationFlags) -> Result<(), Overflow> {
        // Apply gravity
        for first in 0..self.size {
            for second in 0..first {
                const UNIT_VELOCITY: Velocity = Velocity(1);
                match self.position[first].cmp(&self.position[second]) {
                    Ordering::Less => {
                        self.velocity[first] = self.velocity[first].add(UNIT_VELOCITY)?;
                        self.velocity[second] = self.velocity[second].sub(UNIT_VELOCITY)?;
                    }
                    Ordering::Greater => {
                        self.velocity[first] = self.velocity[first].sub(UNIT_VELOCITY)?;
                        self.velocity[second] = self.velocity[second].add(UNIT_VELOCITY)?;
                    }
                    Ordering::Equal => (),
                }
            }
        }

        // Apply velocity
        for i in 0..self.size {
            self.position[i] = self.position[i].add(self.velocity[i])?;
        }
        Ok(())
    }

    fn potential_energy(&self, i: usize) -> i32 {
        self.position[i].0.abs()
    }

    fn kinetic_energy(&self, i: usize) -> i32 {
        self.velocity[i].0.abs()
    }
}

struct System3 {
    systems: [System1D; DIMENSIONS],
    body_count: usize,
}

impl System3 {
    fn new(systems: [System1D; DIMENSIONS]) -> System3 {
        assert!(DIMENSIONS > 0);
        let body_count = systems[0].body_count();
        for i in 0..DIMENSIONS {
            assert_eq!(systems[i].body_count(), body_count);
        }
        System3 {
            systems,
            body_count,
        }
    }

    fn step(&mut self, step_number: usize, flags: &SimulationFlags) -> Result<(), Overflow> {
        for system in self.systems.iter_mut() {
            system.step(flags)?;
        }
        if flags.verbose {
            println!(
                "After {} {}:\n{}",
                step_number,
                if step_number == 1 { "step" } else { "steps" },
                self
            );
        }
        Ok(())
    }

    fn potential_energy(&self, i: usize) -> i32 {
        self.systems.iter().map(|sys| sys.potential_energy(i)).sum()
    }

    fn kinetic_energy(&self, i: usize) -> i32 {
        self.systems.iter().map(|sys| sys.kinetic_energy(i)).sum()
    }

    fn total_energy(&self) -> i32 {
        (0..self.body_count)
            .map(|i| {
                let pot = self.potential_energy(i);
                let kin = self.kinetic_energy(i);
                println!(
                    "Body {} has potential energy {}, kinetic energy {}",
                    i, &pot, &kin
                );
                pot * kin
            })
            .sum()
    }
}

impl Display for System3 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        for body in 0..self.body_count {
            // pos=<x= -8, y=-10, z=  0>, vel=<x=  0, y=  0, z=  0>
            write!(
                f,
                "pos=<x={:>3}, y={:>3}, z={:>3}>, vel=<x={:>3}, y={:>3}, z={:>3}>\n",
                self.systems[0].position[body],
                self.systems[1].position[body],
                self.systems[2].position[body],
                self.systems[0].velocity[body],
                self.systems[1].velocity[body],
                self.systems[2].velocity[body]
            )?;
        }
        Ok(())
    }
}

fn err_as_bad_input<E: std::error::Error>(e: E) -> BadInput {
    BadInput(e.to_string())
}

fn parse_initial_state<S>(lines: &[S]) -> Result<System3, BadInput>
where
    S: AsRef<str>,
{
    let mut initial_positions: Vec<Vec<Distance>> = Vec::new();
    for _ in 0..DIMENSIONS {
        initial_positions.push(Vec::new());
    }
    let extractor = IntegerExtractor::new();
    for (i, line) in lines.iter().enumerate() {
        let line = line.as_ref();
        let values: Vec<i32> = extractor
            .get_integers::<i32, _>(&line)
            .map_err(err_as_bad_input)?;
        if values.len() != DIMENSIONS {
            return Err(BadInput(format!(
                "line {}: expected {} fields, got {}: {}",
                (i + 1),
                DIMENSIONS,
                values.len(),
                &line
            )));
        }
        for dimension in 0..DIMENSIONS {
            initial_positions[dimension].push(Distance(values[dimension]));
        }
    }
    let mut initial_velocities: Vec<Vec<Velocity>> = (0..DIMENSIONS).map(|_| Vec::new()).collect();
    for dimension in 0..DIMENSIONS {
        let body_count = initial_positions[dimension].len();
        initial_velocities[dimension].resize(body_count, Velocity(0));
    }

    Ok(System3::new([
        System1D::new(&initial_positions[0], &initial_velocities[0]),
        System1D::new(&initial_positions[1], &initial_velocities[1]),
        System1D::new(&initial_positions[2], &initial_velocities[2]),
    ]))
}

fn read_initial_state() -> Result<System3, BadInput> {
    let lines: Vec<String> = read_stdin_lines().map_err(err_as_bad_input)?;
    parse_initial_state(&lines)
}

fn solve1(system: &mut System3, steps: usize, flags: &SimulationFlags) -> Result<i32, Overflow> {
    if flags.verbose {
        println!("After 0 steps:\n{}", system);
    }
    for step_number in 1..=steps {
        system.step(step_number, flags)?;
    }
    Ok(system.total_energy())
}

#[test]
fn test_solve1_first_example() {
    let input: Vec<String> = vec![
        "<x=-1, y=0, z=2>\n",
        "<x=2, y=-10, z=-7>\n",
        "<x=4, y=-8, z=8>\n",
        "<x=3, y=5, z=-1>\n",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    let mut system = parse_initial_state(&input).expect("test input should be valid");
    let flags = SimulationFlags { verbose: true };
    let energy = solve1(&mut system, 10, &flags).expect("simulation should succeed");
    assert_eq!(energy, 179);
}

#[test]
fn test_solve1_second_example() {
    let input: Vec<String> = vec![
        "<x=-8, y=-10, z=0>\n",
        "<x=5, y=5, z=10>\n",
        "<x=2, y=-7, z=3>\n",
        "<x=9, y=-8, z=-3>\n",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    let mut system = parse_initial_state(&input).expect("test input should be valid");
    let flags = SimulationFlags { verbose: false };
    let energy = solve1(&mut system, 100, &flags).expect("simulation should succeed");
    assert_eq!(energy, 1940);
}

fn part1(system: &mut System3) {
    const STEPS: usize = 1000;
    let flags = SimulationFlags { verbose: false };
    match solve1(system, STEPS, &flags) {
        Ok(energy) => {
            println!(
                "Day 12 part 1: total energy after {} steps: {}",
                STEPS, energy
            );
        }
        Err(e) => {
            eprintln!("Day 12 part 1: failed: {}", e);
        }
    }
}

fn run() -> Result<(), BadInput> {
    let mut system = read_initial_state()?;
    part1(&mut system);
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("failed: {}", e);
    }
}
