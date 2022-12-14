use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Add, Div, Mul, Rem};
use std::str::FromStr;

use regex::Regex;

use lib::error::Fail;
use lib::input::{read_file_as_lines, run_with_input};

const DIMENSIONS: usize = 3;

#[derive(Debug)]
struct SimulationFlags<FV>
where
    FV: Fn(u64) -> bool,
{
    verbose: FV,
}

#[derive(Debug)]
struct Overflow {}

impl PartialEq for Overflow {
    fn eq(&self, _: &Overflow) -> bool {
        true
    }
}

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

#[derive(Clone)]
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
            position: positions.to_vec(),
            velocity: velocities.to_vec(),
            size: positions.len(),
        }
    }

    fn step<FV>(&mut self, _: &SimulationFlags<FV>) -> Result<(), Overflow>
    where
        FV: Fn(u64) -> bool,
    {
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

    fn axis_match(&self, other: &System1D) -> bool {
        (0..self.size)
            .all(|n| self.position[n] == other.position[n] && self.velocity[n] == other.velocity[n])
    }
}

#[derive(Clone)]
struct System3 {
    systems: [System1D; DIMENSIONS],
    body_count: usize,
}

impl System3 {
    fn new(systems: [System1D; DIMENSIONS]) -> System3 {
        let body_count = systems[0].body_count();
        assert!(systems
            .iter()
            .all(|system| system.body_count() == body_count));
        System3 {
            systems,
            body_count,
        }
    }

    fn step<FV>(&mut self, step_number: u64, flags: &SimulationFlags<FV>) -> Result<(), Overflow>
    where
        FV: Fn(u64) -> bool,
    {
        for system in self.systems.iter_mut() {
            system.step(flags)?;
        }
        if (flags.verbose)(step_number) {
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

    fn axis_match(&self, axis: usize, initial: &System3) -> bool {
        self.systems[axis].axis_match(&initial.systems[axis])
    }
}

impl Display for System3 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        for body in 0..self.body_count {
            // pos=<x= -8, y=-10, z=  0>, vel=<x=  0, y=  0, z=  0>
            writeln!(
                f,
                "pos=<x={:>3}, y={:>3}, z={:>3}>, vel=<x={:>3}, y={:>3}, z={:>3}>",
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

fn parse_initial_state<S>(lines: &[S]) -> Result<System3, Fail>
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
            .map_err(|e| Fail(e.to_string()))?;
        if values.len() != DIMENSIONS {
            return Err(Fail(format!(
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

fn solve1<FV>(
    system: &mut System3,
    steps: u64,
    flags: &SimulationFlags<FV>,
) -> Result<i32, Overflow>
where
    FV: Fn(u64) -> bool,
{
    if (flags.verbose)(0) {
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
    let flags = SimulationFlags { verbose: |_| true };
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
    let flags = SimulationFlags { verbose: |_| false };
    let energy = solve1(&mut system, 100, &flags).expect("simulation should succeed");
    assert_eq!(energy, 1940);
}

fn part1(system: &mut System3) -> Result<(), Fail> {
    const STEPS: u64 = 1000;
    let flags = SimulationFlags { verbose: |_| false };
    match solve1(system, STEPS, &flags) {
        Ok(energy) => {
            println!(
                "Day 12 part 1: total energy after {} steps: {}",
                STEPS, energy
            );
            Ok(())
        }
        Err(e) => Err(Fail(format!("Day 12 part 1: failed: {}", e))),
    }
}

fn gcd<T>(a: T, b: T) -> T
where
    T: Add + Rem<Output = T> + PartialEq + From<u8> + Copy,
{
    if a == 0_u8.into() {
        b
    } else {
        gcd(b % a, a)
    }
}

#[test]
fn test_gcd() {
    assert_eq!(gcd(12_u8, 9_u8), 3_u8);
    assert_eq!(gcd(12_u8, 8_u8), 4_u8);
    assert_eq!(gcd(12_u8, 11_u8), 1_u8);
}

fn lcm<T>(a: T, b: T) -> T
where
    T: Add + Rem<Output = T> + Mul<Output = T> + Div<Output = T> + PartialEq + From<u8> + Copy,
{
    (a * b) / gcd(a, b)
}

#[test]
fn test_lcm() {
    assert_eq!(lcm(3_u8, 4_u8), 12_u8);
    assert_eq!(lcm(12_u8, 8_u8), 24_u8);
}

fn lcm3<T>(a: T, b: T, c: T) -> T
where
    T: Add + Rem<Output = T> + Mul<Output = T> + Div<Output = T> + PartialEq + From<u8> + Copy,
{
    lcm(a, lcm(b, c))
}

fn solve2<FV>(
    system: &mut System3,
    step_limit: u64,
    flags: &SimulationFlags<FV>,
) -> Result<Option<u64>, Overflow>
where
    FV: Fn(u64) -> bool,
{
    let initial = system.clone();
    let mut cycles_to_find: usize = DIMENSIONS;
    let mut cycle: [Option<u64>; DIMENSIONS] = [None, None, None];
    for step_number in 1..=step_limit {
        if cycles_to_find == 0 {
            break;
        }
        system.step(step_number, flags)?;
        for (axis, cyc) in cycle
            .iter_mut()
            .enumerate()
            .filter(|(_, cyc)| cyc.is_none())
        {
            if system.axis_match(axis, &initial) {
                *cyc = Some(step_number);
                cycles_to_find -= 1;
                println!(
                    "solve2: at iteration {} found cycle in dimension {}",
                    step_number, axis
                );
            }
        }
    }
    match (cycle[0], cycle[1], cycle[2]) {
        (Some(a), Some(b), Some(c)) => {
            let full_cycle = lcm3(a, b, c);
            println!("Cycle length on all dimensions is {}", full_cycle);
            Ok(Some(full_cycle))
        }
        _ => {
            eprintln!(
                "Did not find a cycle on at least one dimension: {:?}",
                cycle
            );
            Ok(None)
        }
    }
}

fn part2(system: &mut System3) -> Result<(), Fail> {
    let flags = SimulationFlags { verbose: |_| false };
    match solve2(system, 1000000, &flags) {
        Ok(Some(n)) => {
            println!("Day 12 part 2: {}", n);
            Ok(())
        }
        Ok(_) => Err(Fail("Day 12 part 2: no solution".to_string())),
        Err(e) => Err(Fail(format!("Day 12 part 2: failed: {}", e))),
    }
}

#[test]
fn test_solve2_first_example() {
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
    let flags = SimulationFlags {
        verbose: |n| match n {
            0 | 2770 | 2771 | 2772 => true,
            _ => false,
        },
    };
    assert_eq!(solve2(&mut system, 3000, &flags), Ok(Some(2772)));
}

fn run(lines: Vec<String>) -> Result<(), Fail> {
    let mut system = parse_initial_state(&lines)?;
    part1(&mut system.clone())?;
    part2(&mut system)?;
    Ok(())
}

fn main() -> Result<(), Fail> {
    run_with_input(12, read_file_as_lines, run)
}
