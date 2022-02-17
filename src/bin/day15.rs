use pancurses::{endwin, initscr, Window};
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::{self, Display, Formatter};
use std::thread;
use std::time::Duration;

use lib::cpu::Processor;
use lib::cpu::Word;
use lib::cpu::{read_program_from_stdin, CpuFault, CpuStatus, InputOutputError};

mod grid {
    use std::fmt::{self, Display, Formatter};

    #[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
    pub enum CompassDirection {
        North,
        South,
        West,
        East,
    }

    impl CompassDirection {
        pub fn reversed(&self) -> CompassDirection {
            use CompassDirection::*;
            match self {
                North => South,
                South => North,
                East => West,
                West => East,
            }
        }
    }

    impl From<CompassDirection> for char {
        fn from(d: CompassDirection) -> char {
            use CompassDirection::*;
            match d {
                North => 'N',
                East => 'E',
                South => 'S',
                West => 'W',
            }
        }
    }

    pub const ALL_MOVE_OPTIONS: [CompassDirection; 4] = [
        CompassDirection::North,
        CompassDirection::East,
        CompassDirection::South,
        CompassDirection::West,
    ];

    #[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
    pub struct Position {
        pub x: i64,
        pub y: i64,
    }

    impl Display for Position {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            write!(f, "{},{}", self.x, self.y)
        }
    }

    impl Position {
        pub fn move_direction(&self, d: &CompassDirection) -> Position {
            match d {
                CompassDirection::North => Position {
                    y: self.y - 1,
                    ..*self
                },
                CompassDirection::South => Position {
                    y: self.y + 1,
                    ..*self
                },
                CompassDirection::East => Position {
                    x: self.x + 1,
                    ..*self
                },
                CompassDirection::West => Position {
                    x: self.x - 1,
                    ..*self
                },
            }
        }
    }

    pub fn bounds<'a, I>(points: I) -> Option<(Position, Position)>
    where
        I: IntoIterator<Item = &'a Position>,
    {
        let mut min_x: Option<i64> = None;
        let mut max_x: Option<i64> = None;
        let mut min_y: Option<i64> = None;
        let mut max_y: Option<i64> = None;
        fn maybe_update_min(min: &mut Option<i64>, val: i64) {
            match min {
                None => {
                    *min = Some(val);
                }
                Some(v) if *v > val => *min = Some(val),
                Some(_) => (),
            }
        }
        fn maybe_update_max(max: &mut Option<i64>, val: i64) {
            match max {
                None => {
                    *max = Some(val);
                }
                Some(v) if *v < val => *max = Some(val),
                Some(_) => (),
            }
        }
        for p in points.into_iter() {
            maybe_update_min(&mut min_x, p.x);
            maybe_update_max(&mut max_x, p.x);
            maybe_update_min(&mut min_y, p.y);
            maybe_update_max(&mut max_y, p.y);
        }
        match (min_x, max_x, min_y, max_y) {
            (Some(xlow), Some(xhigh), Some(ylow), Some(yhigh)) => {
                let min: Position = Position { x: xlow, y: ylow };
                let max: Position = Position { x: xhigh, y: yhigh };
                Some((min, max))
            }
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
enum RoomType {
    Wall,
    Open(bool),
    Goal,
    Start,
}

impl From<RoomType> for char {
    fn from(rt: RoomType) -> char {
        match rt {
            RoomType::Start => 'S',
            RoomType::Wall => '#',
            RoomType::Open(filled) => {
                if filled {
                    'O'
                } else {
                    '.'
                }
            }
            RoomType::Goal => 'X',
        }
    }
}

#[derive(Debug)]
struct BadMap(String);

impl TryFrom<char> for RoomType {
    type Error = BadMap;
    fn try_from(ch: char) -> Result<RoomType, BadMap> {
        match ch {
            'S' => Ok(RoomType::Start),
            '#' => Ok(RoomType::Wall),
            'O' => Ok(RoomType::Open(true)),
            '.' => Ok(RoomType::Open(false)),
            ' ' => Ok(RoomType::Wall),
            'X' => Ok(RoomType::Goal),
            _ => Err(BadMap(format!("unexpected character '{}'", ch))),
        }
    }
}

#[derive(Clone, Debug)]
struct Movements {
    steps: Vec<CompassDirection>,
}

impl Movements {
    fn empty() -> Movements {
        Movements { steps: Vec::new() }
    }

    fn len(&self) -> usize {
        self.steps.len()
    }

    fn push_step(&mut self, step: &CompassDirection) {
        self.steps.push(*step);
    }

    fn pop(&mut self) -> Option<CompassDirection> {
        self.steps.pop()
    }

    fn compute_path_locations(&self, origin: &Position) -> Vec<Position> {
        self.steps
            .iter()
            .fold((*origin, vec![*origin]), |(here, mut path), direction| {
                let next = here.move_direction(direction);
                path.push(next);
                (next, path)
            })
            .1
    }
}

use grid::{CompassDirection, Position, ALL_MOVE_OPTIONS};

struct ShipMap {
    tiles: HashMap<grid::Position, RoomType>,
    goal: Option<Position>,
}

impl ShipMap {
    fn new(start: Position) -> ShipMap {
        let mut tiles = HashMap::new();
        tiles.insert(start, RoomType::Start);
        ShipMap { tiles, goal: None }
    }

    fn add_location(&mut self, pos: Position, t: RoomType) {
        if t == RoomType::Goal {
            self.goal = Some(pos);
        }
        self.tiles.insert(pos, t);
    }

    fn oxygen_fill(&mut self, pos: Position) {
        if let Some(RoomType::Open(filled)) = self.tiles.get_mut(&pos) {
            *filled = true;
        }
    }

    fn get_location_type(&self, pos: &Position) -> Option<&RoomType> {
        match self.goal.as_ref() {
            Some(g) if g == pos => Some(&RoomType::Goal),
            _ => self.tiles.get(pos),
        }
    }

    fn get_open_rooms(&self) -> HashSet<Position> {
        self.tiles
            .iter()
            .filter_map(|(pos, room_type)| match room_type {
                RoomType::Start | RoomType::Open(_) | RoomType::Goal => Some(*pos),
                RoomType::Wall => None,
            })
            .collect()
    }

    fn options_from(&self, pos: &Position) -> Vec<CompassDirection> {
        ALL_MOVE_OPTIONS
            .iter()
            .filter(|direction| !self.tiles.contains_key(&pos.move_direction(direction)))
            .copied()
            .collect()
    }

    fn is_known_to_be_the_goal(&self, pos: &Position) -> bool {
        self.goal.as_ref().map(|p| p == pos).unwrap_or(false)
    }

    fn display(&self, w: &mut Window, start: &Position, path: &Movements) {
        const HALF_WIDTH: i64 = 30;
        const HALF_HEIGHT: i64 = 30;
        let path_locations: HashSet<Position> =
            path.compute_path_locations(start).into_iter().collect();
        for y in (-HALF_HEIGHT)..(HALF_HEIGHT - 1) {
            let row: String = ((-HALF_WIDTH)..(HALF_WIDTH - 1))
                .map(|x: i64| -> char {
                    let here = Position { x, y };
                    if x == 0 && y == 0 {
                        '@' // the droid
                    } else if path_locations.contains(&here) {
                        '*'
                    } else {
                        self.get_location_type(&here)
                            .map(|t| (*t).into())
                            .unwrap_or(' ')
                    }
                })
                .collect();
            match (y + HALF_HEIGHT + 1).try_into() {
                Ok(screen_row) => {
                    w.mvprintw(screen_row, 0, row);
                }
                Err(_) => {
                    panic!("unexpected screen_row overflow");
                }
            }
        }
        w.refresh();
    }
}

impl Display for ShipMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match grid::bounds(self.tiles.keys().chain(self.goal.iter())) {
            Some((min, max)) => {
                for y in min.y..=max.y {
                    let row: String = (min.x..=max.x)
                        .map(|x: i64| -> char {
                            let here = Position { x, y };
                            if x == 0 && y == 0 {
                                '@' // the droid
                            } else {
                                self.get_location_type(&here)
                                    .map(|t| (*t).into())
                                    .unwrap_or(' ')
                            }
                        })
                        .collect();
                    writeln!(f, "{}", row)?;
                }
                Ok(())
            }
            None => {
                // Empty; nothing to display.
                Ok(())
            }
        }
    }
}

impl TryFrom<&str> for ShipMap {
    type Error = BadMap;
    fn try_from(s: &str) -> Result<ShipMap, BadMap> {
        let mut result = ShipMap::new(Position { x: 0, y: 0 });
        for (y, line) in s.split('\n').enumerate() {
            for (x, ch) in line.chars().enumerate() {
                let t: RoomType = RoomType::try_from(ch)?;
                result.add_location(
                    Position {
                        x: x as i64,
                        y: y as i64,
                    },
                    t,
                );
            }
        }
        Ok(result)
    }
}

struct MoveResult {
    moved: bool,
    new_location: Position,
    cpu_status: CpuStatus,
}

struct RepairDroid {
    cpu: Processor,
}

impl RepairDroid {
    fn new(program: &[Word]) -> Result<RepairDroid, CpuFault> {
        let mut cpu = Processor::new(Word(0));
        cpu.load(Word(0), program)?;
        Ok(RepairDroid { cpu })
    }

    fn move_droid(
        &mut self,
        current_position: &Position,
        which_way: &CompassDirection,
        ship_map: &mut ShipMap,
    ) -> Result<MoveResult, CpuFault> {
        enum RunResult {
            Running(Word),
            Stopped,
        }
        let mut run_until_output = |which_way: &CompassDirection| -> Result<RunResult, CpuFault> {
            let mut input_word: Option<Word> = Some(match which_way {
                CompassDirection::North => Word(1),
                CompassDirection::South => Word(2),
                CompassDirection::West => Word(3),
                CompassDirection::East => Word(4),
            });
            let mut do_input = || -> Result<Word, InputOutputError> {
                match input_word.take() {
                    Some(w) => Ok(w),
                    None => {
                        panic!("RepairDroid::move: program read more than one input word");
                    }
                }
            };

            loop {
                let mut output_word: Option<Word> = None;
                let mut do_output = |w: Word| -> Result<(), InputOutputError> {
                    output_word = Some(w);
                    Ok(())
                };
                match self.cpu.execute_instruction(&mut do_input, &mut do_output) {
                    Err(e) => return Err(e),
                    Ok(CpuStatus::Halt) => return Ok(RunResult::Stopped),
                    Ok(CpuStatus::Run) => (),
                }
                if let Some(w) = output_word.as_ref() {
                    return Ok(RunResult::Running(*w));
                }
            }
        };

        let target = current_position.move_direction(which_way);
        match run_until_output(which_way) {
            Err(e) => Err(e),
            Ok(RunResult::Stopped) => Ok(MoveResult {
                cpu_status: CpuStatus::Halt,
                moved: false,
                new_location: *current_position,
            }),
            Ok(RunResult::Running(w)) => match w {
                Word(0) => {
                    ship_map.add_location(target, RoomType::Wall);
                    Ok(MoveResult {
                        cpu_status: CpuStatus::Run,
                        moved: false,
                        new_location: *current_position,
                    })
                }
                Word(1 | 2) => {
                    ship_map.add_location(
                        target,
                        if w == Word(1) {
                            RoomType::Open(false)
                        } else {
                            RoomType::Goal
                        },
                    );
                    Ok(MoveResult {
                        cpu_status: CpuStatus::Run,
                        moved: true,
                        new_location: target,
                    })
                }
                other => {
                    panic!("program generated unexpected output {}", other);
                }
            },
        }
    }
}

fn shortest_path_to_goal(
    start: &Position,
    current_position: &Position,
    mut current_path: Movements,
    droid: &mut RepairDroid,
    ship_map: &mut ShipMap,
    window: &mut Window,
) -> Result<Option<Movements>, CpuFault> {
    ship_map.display(window, start, &current_path);
    if ship_map.is_known_to_be_the_goal(current_position) {
        return Ok(Some(current_path.clone()));
    }
    let mut best_path: Option<Movements> = None;
    for direction in ship_map.options_from(current_position) {
        match droid.move_droid(current_position, &direction, ship_map)? {
            MoveResult {
                cpu_status: CpuStatus::Halt,
                ..
            } => {
                panic!("droid CPU halted during move");
            }
            MoveResult {
                moved: false,
                cpu_status: CpuStatus::Run,
                ..
            } => (),
            MoveResult {
                moved: true,
                new_location,
                cpu_status: CpuStatus::Run,
            } => {
                current_path.push_step(&direction);
                match (
                    best_path.as_ref(),
                    shortest_path_to_goal(
                        start,
                        &new_location,
                        current_path.clone(),
                        droid,
                        ship_map,
                        window,
                    )?,
                ) {
                    (_, None) => (),
                    (None, Some(new_path)) => {
                        best_path = Some(new_path);
                    }
                    (Some(existing), Some(new_path)) => {
                        if new_path.len() < existing.len() {
                            best_path = Some(new_path);
                        }
                    }
                }
                let before_retracing_steps: Position = new_location;
                match droid.move_droid(&new_location, &direction.reversed(), ship_map)? {
                    MoveResult {
                        cpu_status: CpuStatus::Halt,
                        ..
                    } => {
                        panic!("droid CPU halted while retracing steps");
                    }
                    MoveResult {
                        cpu_status: CpuStatus::Run,
                        new_location,
                        ..
                    } => {
                        current_path.pop();
                        if new_location == before_retracing_steps {
                            panic!("droid hit a wall where we don't think there is a wall");
                        } else if &new_location != current_position {
                            panic!("droid went in an unexpected direction when retracing steps");
                        }
                    }
                }
            }
        }
    }
    Ok(best_path)
}

fn part1(
    start: &Position,
    droid: &mut RepairDroid,
    window: &mut Window,
) -> Result<Option<(ShipMap, usize)>, CpuFault> {
    let mut ship_map = ShipMap::new(*start);
    let result = shortest_path_to_goal(
        start,
        start,
        Movements::empty(),
        droid,
        &mut ship_map,
        window,
    );
    if let Ok(Some(shortest)) = result.as_ref() {
        ship_map.display(window, start, shortest);
    }
    window.mvprintw(0, 0, "** FINISHED : PRESS A KEY TO CONTINUE **");
    window.refresh();
    thread::sleep(Duration::from_millis(4000));
    window.getch();
    match result {
        Err(e) => Err(e),
        Ok(Some(path)) => Ok(Some((ship_map, path.len()))),
        Ok(None) => {
            eprintln!("Day 15 part 1: did not find a solution");
            Ok(None)
        }
    }
}

fn advance(boundary: &HashSet<Position>, open_rooms: &HashSet<Position>) -> HashSet<Position> {
    boundary
        .iter()
        .flat_map(|pos| {
            ALL_MOVE_OPTIONS.iter().filter_map(|direction| {
                let next_pos = pos.move_direction(direction);
                if open_rooms.contains(&next_pos) {
                    Some(next_pos)
                } else {
                    None
                }
            })
        })
        .collect()
}

fn part2<F>(start: &Position, ship_map: &mut ShipMap, mut display_state: F) -> usize
where
    F: FnMut(usize, usize, &ShipMap),
{
    let mut boundary: HashSet<Position> = HashSet::new();
    let mut to_fill: HashSet<Position> = ship_map.get_open_rooms();
    boundary.insert(*start);
    to_fill.remove(start);
    let mut fill_count: usize = 1;
    for step_number in 0.. {
        display_state(step_number, fill_count, ship_map);
        if to_fill.is_empty() {
            return step_number;
        }
        let new_boundary: HashSet<Position> = advance(&boundary, &to_fill);
        for filled_pos in new_boundary.iter() {
            ship_map.oxygen_fill(*filled_pos);
            to_fill.remove(filled_pos);
            fill_count += 1;
        }
        boundary = new_boundary;
    }
    unreachable!()
}

#[test]
fn test_part2() {
    let mut sm = ShipMap::try_from(concat!(
        " ##   \n", "#..## \n", "#.#..#\n", "#.X.# \n", " ###  \n",
    ))
    .expect("test input should be valid");
    let oxy = Position { x: 2, y: 3 };
    let display_map = |step: usize, occupied: usize, sm: &ShipMap| {
        println!("Step {}: {} cells occupied:\n{}", step, occupied, sm);
    };
    assert_eq!(part2(&oxy, &mut sm, display_map), 4);
}

fn run(program: &[Word]) -> Result<(), CpuFault> {
    let start = Position { x: 0, y: 0 };
    let mut droid = RepairDroid::new(program)?;
    let mut window = initscr();
    let result_msg: Result<String, CpuFault> = match part1(&start, &mut droid, &mut window) {
        Ok(Some((mut ship_map, part1_path_len))) => match ship_map.goal {
            Some(g) => {
                let empty_movements: Movements = Movements::empty();
                let step = part2(
                    &g,
                    &mut ship_map,
                    |_step: usize, _occ: usize, map: &ShipMap| {
                        map.display(&mut window, &g, &empty_movements)
                    },
                );
                endwin();
                Ok(format!(
                    "Day 15 part 1: path length is {}\nDay 15 part 2: fill at step {}",
                    part1_path_len, step
                ))
            }
            None => {
                panic!("no oxygen system");
            }
        },
        Ok(None) => Ok("Day 15: no solution found to part 1".to_string()),
        Err(e) => Err(e),
    };
    endwin();
    match result_msg {
        Ok(msg) => {
            println!("{}", msg);
            Ok(())
        }
        Err(e) => Err(e),
    }
}

fn main() {
    match read_program_from_stdin() {
        Ok(words) => {
            run(&words).expect("program should not fail");
        }
        Err(e) => {
            eprintln!("failed to load program: {}", e);
        }
    }
}
