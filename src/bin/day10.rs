use std::cmp::Ordering;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::io::{self, Read};

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Hash, Clone)]
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn colinear_triple(p1: &Point, p2: &Point, p3: &Point) -> bool {
        let a = p1.x * (p2.y - p3.y) + p2.x * (p3.y - p1.y) + p3.x * (p1.y - p2.y);
        a == 0
    }

    fn same_side_of_p(&self, q: &Point, r: &Point) -> bool {
        let xq = q.x - self.x;
        let yq = q.y - self.y;
        let xr = r.x - self.x;
        let yr = r.y - self.y;
        (xq > 0) == (xr > 0) && (yq > 0) == (yr > 0)
    }

    fn manhattan(&self, other: &Point) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }

    fn furthest_point<'a>(&self, q: &'a Point, r: &'a Point) -> &'a Point {
        if self.manhattan(q) > self.manhattan(r) {
            q
        } else {
            r
        }
    }
}

#[test]
fn test_colinear() {
    assert!(Point::colinear_triple(
        &Point { x: 0, y: 0 },
        &Point { x: 1, y: 0 },
        &Point { x: 2, y: 0 }
    ));
    assert!(!Point::colinear_triple(
        &Point { x: 0, y: 0 },
        &Point { x: 1, y: 0 },
        &Point { x: 2, y: 1 }
    ));
}

#[test]
fn test_same_side_of_p() {
    assert!(!Point { x: 5, y: 8 }.same_side_of_p(&Point { x: 1, y: 7 }, &Point { x: 9, y: 9 }));
}

#[derive(Debug)]
struct AsteroidField {
    asteroids: HashSet<Point>,
}

impl From<&str> for AsteroidField {
    fn from(input: &str) -> AsteroidField {
        let mut asteroids: HashSet<Point> = HashSet::new();
        let mut x = 0;
        let mut y = 0;
        for ch in input.chars() {
            match ch {
                '.' => (), // empty
                '\n' => {
                    y += 1;
                    x = 0;
                    continue;
                }
                _ => {
                    asteroids.insert(Point { x, y });
                }
            }
            x += 1;
        }
        AsteroidField { asteroids }
    }
}

fn parse_input() -> Result<AsteroidField, std::io::Error> {
    let mut input: String = String::new();
    io::BufReader::new(io::stdin()).read_to_string(&mut input)?;
    Ok(input.as_str().into())
}

#[derive(Debug, Eq, PartialEq, Clone)]
struct Candidate {
    p: Point,
    visible_count: usize,
}

impl Ord for Candidate {
    fn cmp(&self, other: &Candidate) -> Ordering {
        match self.visible_count.cmp(&other.visible_count) {
            Ordering::Equal => self.p.cmp(&other.p),
            unequal => unequal,
        }
    }
}

impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Candidate) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn solve1(field: &AsteroidField) -> Option<Candidate> {
    let mut candidates: BTreeSet<Candidate> = BTreeSet::new();
    for p in field.asteroids.iter() {
        let mut maybe_visible_from_p: HashSet<Point> = field
            .asteroids
            .iter()
            .filter(|q| *q != p)
            .cloned()
            .collect();
        let mut invisible_from_p: HashMap<Point, Point> = HashMap::new();
        for q in maybe_visible_from_p.iter() {
            if invisible_from_p.contains_key(q) {
                // Skipping q because a some other point is already
                // between it and p.
                continue;
            }
            for r in maybe_visible_from_p.iter() {
                if r == q || r == p || p == q {
                    continue;
                }
                if invisible_from_p.contains_key(r) {
                    // Skipping r because some other point is already
                    // between it and p.
                    continue;
                }
                if !Point::colinear_triple(p, q, r) {
                    continue;
                }
                if !p.same_side_of_p(q, r) {
                    continue;
                }
                let furthest: &Point = p.furthest_point(q, r);
                let nearest: &Point = if furthest == q { p } else { q };
                invisible_from_p.insert(furthest.clone(), nearest.clone());
                if furthest == q {
                    break;
                }
            }
        }
        for goner in invisible_from_p.keys() {
            maybe_visible_from_p.remove(goner);
        }
        for (occluded, occluder) in invisible_from_p.iter() {
            assert!(Point::colinear_triple(p, occluder, occluded));
            assert!(p.furthest_point(occluder, occluded) == occluded);
        }
        candidates.insert(Candidate {
            p: p.clone(),
            visible_count: maybe_visible_from_p.len(),
        });
    }
    candidates.iter().rev().cloned().next()
}

#[cfg(test)]
fn check_solution1(input: &str, expected_solution: &Candidate) {
    let asteroids: AsteroidField = input.into();
    let best = solve1(&asteroids);
    assert_eq!(Some(expected_solution), best.as_ref());
}

#[test]
fn test_solve1() {
    check_solution1(
        r#".#..#
.....
#####
....#
...##"#,
        &Candidate {
            p: Point { x: 3, y: 4 },
            visible_count: 8,
        },
    );

    check_solution1(
        concat!(
            "......#.#.\n",
            "#..#.#....\n",
            "..#######.\n",
            ".#.#.###..\n",
            ".#..#.....\n",
            "..#....#.#\n",
            "#..#....#.\n",
            ".##.#..###\n",
            "##...#..#.\n",
            ".#....####\n"
        ), // input
        &Candidate {
            p: Point { x: 5, y: 8 },
            visible_count: 33,
        },
    );

    check_solution1(
        concat!(
            "#.#...#.#.\n",
            ".###....#.\n",
            ".#....#...\n",
            "##.#.#.#.#\n",
            "....#.#.#.\n",
            ".##..###.#\n",
            "..#...##..\n",
            "..##....##\n",
            "......#...\n",
            ".####.###.\n"
        ), // input
        &Candidate {
            p: Point { x: 1, y: 2 },
            visible_count: 35,
        },
    );

    check_solution1(
        concat!(
            ".#..#..###\n",
            "####.###.#\n",
            "....###.#.\n",
            "..###.##.#\n",
            "##.##.#.#.\n",
            "....###..#\n",
            "..#.#..#.#\n",
            "#..#.#.###\n",
            ".##...##.#\n",
            ".....#.#..\n"
        ), // input
        &Candidate {
            p: Point { x: 6, y: 3 },
            visible_count: 41,
        },
    );

    check_solution1(
        concat!(
            ".#..##.###...#######\n",
            "##.############..##.\n",
            ".#.######.########.#\n",
            ".###.#######.####.#.\n",
            "#####.##.#.##.###.##\n",
            "..#####..#.#########\n",
            "####################\n",
            "#.####....###.#.#.##\n",
            "##.#################\n",
            "#####.##.###..####..\n",
            "..######..##.#######\n",
            "####.##.####...##..#\n",
            ".#####..#.######.###\n",
            "##...#.##########...\n",
            "#.##########.#######\n",
            ".####.#.###.###.#.##\n",
            "....##.##.###..#####\n",
            ".#.#.###########.###\n",
            "#.#.#.#####.####.###\n",
            "###.##.####.##.#..##\n"
        ),
        &Candidate {
            p: Point { x: 11, y: 13 },
            visible_count: 210,
        },
    );
}

fn run() -> Result<(), std::io::Error> {
    let field: AsteroidField = parse_input()?;
    match solve1(&field) {
        Some(solution) => {
            println!("Day 10 part 1: {:?}", &solution);
        }
        None => {
            println!("Day 10 part 1: no solution found");
        }
    }
    Ok(())
}

fn main() -> Result<(), String> {
    run().map_err(|e| format!("{:?}", e))
}
