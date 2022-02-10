use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::f64::consts::PI;
use std::fmt::Display;

use aoc::read_stdin_as_string;

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

    fn bearing(&self, to: &Point) -> f64 {
        let dx: f64 = (to.x - self.x).into();
        let dy: f64 = (to.y - self.y).into();
        let mut rad = -1.0 * (-dy).atan2(dx) + (PI / 2.0);
        if rad < 0.0 {
            rad += 2.0 * PI;
        }
        radians_to_degrees(rad)
    }
}

impl Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{}", self.x, self.y)
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
    Ok(read_stdin_as_string()?.as_str().into())
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

fn radians_to_degrees(rad: f64) -> f64 {
    180.0 * rad / PI
}

#[cfg(test)]
fn is_close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1.0e-5
}

#[cfg(test)]
fn check_radians_to_degrees(radians: f64, expected: f64) {
    let got = radians_to_degrees(radians);
    assert!(is_close(expected, got), "{} vs {}", expected, got);
}

#[cfg(test)]
fn check_bearing_from(from: &Point, to: &Point, expected: f64) {
    let got = from.bearing(to);
    assert!(
        is_close(got, expected),
        "bearing of {} from {}: expected {}, got {}",
        to,
        from,
        expected,
        got
    );
}

#[test]
fn test_bearing() {
    let base = Point { x: 5, y: 5 };
    let examples = &[
        Point { x: 5, y: 4 },
        Point { x: 6, y: 4 },
        Point { x: 6, y: 5 },
        Point { x: 6, y: 6 },
        Point { x: 5, y: 6 },
        Point { x: 4, y: 6 },
        Point { x: 4, y: 5 },
    ];
    for p in examples {
        let b = base.bearing(p);
        println!("Bearing from {} to {} is {}", base, p, b);
    }

    check_radians_to_degrees(0.0, 0.0);
    check_radians_to_degrees(4.0 * PI / 9.0, 80.0);

    check_bearing_from(&Point { x: 5, y: 5 }, &Point { x: 5, y: 4 }, 0.0);
    check_bearing_from(&Point { x: 5, y: 5 }, &Point { x: 10, y: 5 }, 90.0);
    check_bearing_from(&Point { x: 5, y: 5 }, &Point { x: 5, y: 10 }, 180.0);
    check_bearing_from(&Point { x: 5, y: 5 }, &Point { x: 0, y: 5 }, 270.0);
}

fn order_by_reverse_distance(base: &Point, points: &mut Vec<Point>) {
    // We already know tha the slopes of the line betwen base and a is the
    // same as the slope of the line between base and b.  Hence to find the
    // closer of a and b we can simply use the manhattan distance.
    points
        .sort_by(|a: &Point, b: &Point| -> Ordering { base.manhattan(b).cmp(&base.manhattan(a)) });
}

fn solve2(index: usize, base: &Point, asteroids: &AsteroidField) -> Option<Point> {
    const BEARING_MULTIPLIER: f64 = 1.0e6;
    let mut by_direction: BTreeMap<i64, Vec<Point>> = BTreeMap::new();
    for asteroid in asteroids.asteroids.iter() {
        if asteroid != base {
            // The slope calculation is unfamiliar here because y=0 is at the top.
            let b = base.bearing(asteroid);
            println!(
                "The angle in degrees between {} and {} is {}",
                base, asteroid, b
            );
            let bi = (b * BEARING_MULTIPLIER).round() as i64;
            by_direction
                .entry(bi)
                .or_insert_with(Vec::new)
                .push(asteroid.clone());
        }
    }

    for (_bearing, points) in by_direction.iter_mut() {
        order_by_reverse_distance(base, points);
        if points.len() > 1 {
            print!("Order by distance (far to near) from {}:", base);
            for p in points.iter() {
                print!(" {}", p);
            }
            println!();
        }
    }

    let mut zapped: usize = 0;
    let total: usize = by_direction.values().map(|v| v.len()).sum();
    if total < index {
        println!(
            "There can be no {}th asteroid beign zapped, as there are only {} asteroids",
            index, total
        );
        return None;
    }

    println!("The monitoring station is at {}", base);
    loop {
        // The laser starts by pointing up.  So, iterate in order (so
        // that we start at 0 ("up") and move clockwise).
        for (bearing, asteroid_locations) in by_direction.iter_mut() {
            println!(
                "Aiming laser with slope {}",
                (*bearing as f64) / BEARING_MULTIPLIER
            );
            if let Some(goner) = asteroid_locations.pop() {
                zapped += 1;
                println!("Zap asteroid {} at {}", zapped, goner);
                if zapped == index {
                    return Some(goner);
                }
            }
        }
    }
}

#[test]
fn test_solve2() {
    let asteroids: AsteroidField = concat!(
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
    )
    .into();
    let base = Point { x: 11, y: 13 };
    assert_eq!(Some(Point { x: 11, y: 12 }), solve2(1, &base, &asteroids));
    assert_eq!(Some(Point { x: 8, y: 2 }), solve2(200, &base, &asteroids));
    assert_eq!(Some(Point { x: 11, y: 1 }), solve2(299, &base, &asteroids));
}

fn run() -> Result<(), std::io::Error> {
    let field: AsteroidField = parse_input()?;
    match solve1(&field) {
        Some(solution) => {
            println!("Day 10 part 1: {:?}", &solution);

            match solve2(200, &solution.p, &field) {
                Some(asteroid) => {
                    let answer = asteroid.x * 100 + asteroid.y;
                    println!("Day 10 part 2: {}", answer);
                }
                None => {
                    println!("Day 10 part 2: no solution found");
                }
            }
        }
        None => {
            println!("Day 10 part 1: no solution found (so can't solve part 2 either)");
        }
    };
    Ok(())
}

fn main() -> Result<(), String> {
    run().map_err(|e| format!("{:?}", e))
}
