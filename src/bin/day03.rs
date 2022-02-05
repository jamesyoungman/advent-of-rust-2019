use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::io;
use std::io::BufRead;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    const fn origin() -> Point {
        Point { x: 0, y: 0 }
    }

    fn manhattan_from_origin(&self) -> i32 {
        self.x.abs() + self.y.abs()
    }

    fn advance_in_direction(self, m: &Move) -> Point {
        Point {
            x: self.x + m.xdelta.signum(),
            y: self.y + m.ydelta.signum(),
        }
    }
}

impl Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{}", self.x, self.y)
    }
}

#[derive(Debug)]
struct Move {
    xdelta: i32,
    ydelta: i32,
    distance: i32,
}

#[derive(Debug)]
struct BadMove(String);

impl Display for BadMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.as_str())
    }
}

impl TryFrom<&str> for Move {
    type Error = BadMove;
    fn try_from(s: &str) -> Result<Move, BadMove> {
        fn make_xmove(distance: i32) -> Move {
            Move {
                xdelta: distance.signum(),
                ydelta: 0,
                distance: distance.abs(),
            }
        }
        fn make_ymove(distance: i32) -> Move {
            Move {
                xdelta: 0,
                ydelta: distance.signum(),
                distance: distance.abs(),
            }
        }

        match (s.get(0..1), s.get(1..).map(|tail| tail.parse::<i32>())) {
            (Some("L"), Some(Ok(n))) if n >= 0 => Ok(make_xmove(-n)),
            (Some("R"), Some(Ok(n))) if n >= 0 => Ok(make_xmove(n)),
            (Some("U"), Some(Ok(n))) if n >= 0 => Ok(make_ymove(n)),
            (Some("D"), Some(Ok(n))) if n >= 0 => Ok(make_ymove(-n)),
            _ => Err(BadMove(s.to_string())),
        }
    }
}

fn add_move(mut current: Point, this_move: &Move, path: &mut HashSet<Point>) -> Point {
    let origin = Point::origin();
    for _ in 0..this_move.distance {
        if current != origin {
            path.insert(current);
        }
        current = current.advance_in_direction(&this_move);
    }
    current
}

struct Figure {
    symbols: HashMap<Point, char>,
}

impl Figure {
    const PORT: Point = Point::origin();
    fn new() -> Figure {
        let mut symbols = HashMap::new();
        symbols.insert(Self::PORT, 'o');
        Figure { symbols }
    }

    fn draw(
        x: i32,
        y: i32,
        xdelta: i32,
        ydelta: i32,
        first: bool,
        canvas: &mut HashMap<Point, char>,
    ) {
        if x != 0 || y != 0 {
            let symbol = if first {
                '+'
            } else {
                match (xdelta, ydelta) {
                    (0, _) => '|',
                    (_, 0) => '-',
                    _ => {
                        panic!(
                            "move should be horizontal or vertical: {},{}",
                            xdelta, ydelta
                        );
                    }
                }
            };
            println!(
                "Figure::add_move: at {},{}, {},{}: drawing {}",
                x, y, xdelta, ydelta, symbol
            );
            canvas.insert(Point { x, y }, symbol);
        }
    }

    fn add_move(&mut self, mut current: Point, m: &Move) {
        for i in 0..m.distance {
            Self::draw(
                current.x,
                current.y,
                m.xdelta,
                m.ydelta,
                i == 0,
                &mut self.symbols,
            );
            current = current.advance_in_direction(&m);
        }
    }

    fn add_intersections(&mut self, intersections: &HashSet<Point>) {
        for point in intersections.iter() {
            if point != &Self::PORT {
                self.symbols.insert(*point, 'X');
            }
        }
    }
}

impl Display for Figure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.symbols.is_empty() {
            Ok(())
        } else {
            let minx = self.symbols.keys().map(|p| p.x).min().unwrap();
            let maxx = self.symbols.keys().map(|p| p.x).max().unwrap();
            let miny = self.symbols.keys().map(|p| p.y).min().unwrap();
            let maxy = self.symbols.keys().map(|p| p.y).max().unwrap();
            for y in (miny..=maxy).rev() {
                for x in minx..=maxx {
                    let ch: char = match self.symbols.get(&Point { x, y }) {
                        Some(ch) => *ch,
                        None => '.',
                    };
                    write!(f, "{}", ch)?;
                }
                f.write_str("\n")?;
            }
            Ok(())
        }
    }
}

fn make_path(start: &Point, moves: &[Move], fig: &mut Option<Figure>) -> HashSet<Point> {
    let mut result = HashSet::new();
    let mut current = start.clone();
    for this_move in moves {
        if let Some(figure) = fig {
            figure.add_move(current, this_move);
        }
        current = add_move(current, this_move, &mut result);
    }
    result
}

fn solve1(first_path: &[Move], second_path: &[Move], fig: &mut Option<Figure>) -> Option<i32> {
    let origin = Point::origin();
    let path1 = make_path(&origin, first_path, fig);
    let path2 = make_path(&origin, second_path, fig);
    let intersections = path1.intersection(&path2).cloned().collect();
    if let Some(figure) = fig {
        figure.add_intersections(&intersections);
        println!("{}", &figure)
    }
    intersections
        .iter()
        .map(|p| p.manhattan_from_origin())
        .min()
}

#[test]
fn test_solve1() {
    fn check_solution(first: &str, second: &str, expected_dist: i32) {
        let m1: Vec<Move> = string_to_moves(first).expect("first test input should be valid");
        let m2: Vec<Move> = string_to_moves(second).expect("second test input should be valid");
        let mut fig: Option<Figure> = Some(Figure::new());
        match solve1(&m1, &m2, &mut fig) {
            Some(got) if got == expected_dist => (),
            Some(got) => {
                panic!(
                    "{}, {}: expected {}, got {}",
                    first, second, expected_dist, got,
                );
            }
            None => {
                panic!("{}, {}: test case had no solution", first, second);
            }
        }
    }
    check_solution("R8,U5,L5,D3", "U7,R6,D4,L4", 6);
    check_solution(
        "R75,D30,R83,U83,L12,D49,R71,U7,L72",
        "U62,R66,U55,R34,D71,R55,D58,R83",
        159,
    );
    check_solution(
        "R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51",
        "U98,R91,D20,R16,D67,R40,U7,R15,U6,R7",
        135,
    );
}

fn part1(lines: &[Vec<Move>], figure: &mut Option<Figure>) {
    match lines {
        [first, second] => match solve1(&first, &second, figure) {
            Some(d) => {
                println!(
                    "Day 2 part 1: manhattan distance of closest intersection is {}",
                    d
                );
            }
            None => {
                println!("Day 2 part 1: no solution, paths do not intersect");
            }
        },
        _ => {
            panic!("expected 2 paths, got {}", lines.len());
        }
    }
}

fn string_to_moves(s: &str) -> Result<Vec<Move>, BadMove> {
    s.split(',').map(Move::try_from).collect()
}

fn main() {
    let wires: Vec<Vec<Move>> = io::BufReader::new(io::stdin())
        .lines()
        .map(|s| -> Vec<Move> {
            string_to_moves(s.unwrap().as_str()).expect("input should be valid")
        })
        .collect();
    part1(&wires, &mut None);
}
