use pancurses::{endwin, initscr, Window};
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::{self, Display, Formatter};

use cpu::Processor;
use cpu::Word;
use cpu::{read_program_from_stdin, CpuFault, CpuStatus, InputOutputError};

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
enum CompassDirection {
    North,
    South,
    West,
    East,
}

impl CompassDirection {
    fn reversed(&self) -> CompassDirection {
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

const ALL_MOVE_OPTIONS: [CompassDirection; 4] = [
    CompassDirection::North,
    CompassDirection::East,
    CompassDirection::South,
    CompassDirection::West,
];

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
struct Position {
    x: Word,
    y: Word,
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{},{}", self.x, self.y)
    }
}

impl Position {
    fn move_direction(&self, d: &CompassDirection) -> Position {
        match d {
            CompassDirection::North => Position {
                y: Word(self.y.0 - 1),
                ..*self
            },
            CompassDirection::South => Position {
                y: Word(self.y.0 + 1),
                ..*self
            },
            CompassDirection::East => Position {
                x: Word(self.x.0 + 1),
                ..*self
            },
            CompassDirection::West => Position {
                x: Word(self.x.0 - 1),
                ..*self
            },
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
enum RoomType {
    Wall,
    Open,
    Goal,
    Start,
}

impl From<RoomType> for char {
    fn from(rt: RoomType) -> char {
        match rt {
            RoomType::Start => 'S',
            RoomType::Wall => '#',
            RoomType::Open => '.',
            RoomType::Goal => 'X',
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

struct ShipMap {
    tiles: HashMap<Position, RoomType>,
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

    fn get_location_type(&self, pos: &Position) -> Option<&RoomType> {
        match self.goal.as_ref() {
            Some(g) if g == pos => Some(&RoomType::Goal),
            _ => self.tiles.get(pos),
        }
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
            let yw = Word(y);
            let row: String = ((-HALF_WIDTH)..(HALF_WIDTH - 1))
                .map(|x: i64| -> char {
                    let here = Position { x: Word(x), y: yw };
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
                            RoomType::Open
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

fn part1(program: &[Word]) -> Result<(), CpuFault> {
    let start = Position {
        x: Word(0),
        y: Word(0),
    };
    let mut ship_map = ShipMap::new(start);
    let mut droid = RepairDroid::new(program)?;
    let mut window = initscr();
    let result = shortest_path_to_goal(
        &start,
        &start,
        Movements::empty(),
        &mut droid,
        &mut ship_map,
        &mut window,
    );
    if let Ok(Some(shortest)) = result.as_ref() {
        ship_map.display(&mut window, &start, shortest);
    }
    window.mvprintw(0, 0, "** FINISHED : PRESS A KEY TO CONTINUE **");
    window.refresh();
    window.getch();
    endwin();
    match result {
        Err(e) => Err(e),
        Ok(Some(path)) => {
            println!(
                "Day 15 part 1: shortest route has {} moves: {:?}",
                path.len(),
                path
            );
            Ok(())
        }
        Ok(None) => {
            eprintln!("Day 15 part 1: did not find a solution");
            Ok(())
        }
    }
}

fn main() {
    match read_program_from_stdin() {
        Ok(words) => {
            part1(&words).expect("program should not fail in part1");
        }
        Err(e) => {
            eprintln!("failed to load program: {}", e);
        }
    }
}
