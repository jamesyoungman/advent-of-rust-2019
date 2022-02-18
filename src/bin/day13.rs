use pancurses::{endwin, initscr, Window};
use std::cmp::Ordering;
use std::collections::HashSet;
use std::fmt::{self, Display, Formatter};
use std::fs::OpenOptions;
use std::sync::Arc;
use std::sync::Mutex;
use std::{thread, time};

use lib::cpu::{read_program_from_file, CpuFault, InputOutputError, Processor, Word};
use lib::error::Fail;
use lib::input::run_with_input;

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

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
enum DrawCommand {
    DrawTile { pos: Position, tile: Word },
    UpdateScore(Word),
}

struct DisplayCommandInterpreter {
    pending: Vec<Word>,
}

impl DisplayCommandInterpreter {
    fn new() -> DisplayCommandInterpreter {
        DisplayCommandInterpreter {
            pending: Vec::with_capacity(3),
        }
    }

    fn put(&mut self, item: Word) -> Option<DrawCommand> {
        assert!(self.pending.len() < 3);
        self.pending.push(item);
        match self.pending.as_slice() {
            [Word(-1), Word(0), score] => {
                let result = DrawCommand::UpdateScore(*score);
                self.pending.clear();
                Some(result)
            }
            [x, y, tile] => {
                let result = DrawCommand::DrawTile {
                    pos: Position { x: *x, y: *y },
                    tile: *tile,
                };
                self.pending.clear();
                Some(result)
            }
            _ => None,
        }
    }
}

fn part1(program: &[Word]) -> Result<(), CpuFault> {
    fn run(program: &[Word], disp: &mut DisplayCommandInterpreter) -> Result<usize, CpuFault> {
        let mut blocks: HashSet<Position> = HashSet::new();
        let mut get_input = || Ok(Word(0));
        let mut do_output = |w: Word| -> Result<(), InputOutputError> {
            if let Some(DrawCommand::DrawTile { pos, tile: Word(2) }) = disp.put(w) {
                blocks.insert(pos);
            }
            Ok(())
        };
        let mut cpu = Processor::new(Word(0));
        cpu.load(Word(0), program)?;
        cpu.run_with_io(&mut get_input, &mut do_output)?;
        Ok(blocks.len())
    }

    let mut disp_interp = DisplayCommandInterpreter::new();
    let block_count = run(program, &mut disp_interp)?;
    println!("Day 13 part 1: block count is {}", block_count);
    Ok(())
}

struct GameState {
    bat: Word,
    ball: Word,
    score: Word,
    window: Option<Window>,
}

impl GameState {
    fn new() -> GameState {
        GameState {
            bat: Word(0),
            ball: Word(0),
            score: Word(0),
            window: None,
        }
    }

    fn init(&mut self) {
        let w = initscr();
        self.window = Some(w);
    }

    fn done(&mut self) {
        if self.window.is_some() {
            thread::sleep(time::Duration::from_millis(4000));
            endwin();
        }
    }

    fn update_from(&mut self, update: Option<DrawCommand>) {
        match update {
            Some(DrawCommand::UpdateScore(newscore)) => {
                self.score = newscore;
            }
            Some(DrawCommand::DrawTile { pos, tile: Word(3) }) => {
                self.bat = pos.x;
            }
            Some(DrawCommand::DrawTile { pos, tile: Word(4) }) => {
                self.ball = pos.x;
            }
            _ => (),
        }
        if let Some(w) = self.window.as_mut() {
            match update {
                None | Some(DrawCommand::UpdateScore(_)) => (),
                Some(DrawCommand::DrawTile { pos, tile }) => {
                    let symbol: &str = match tile.0 {
                        0 => " ", // empty
                        1 => "|", // wall
                        2 => "#", // block
                        3 => "=", // paddle
                        4 => "o", // ball
                        _ => unreachable!(),
                    };
                    w.mvprintw(pos.y.0 as i32, pos.x.0 as i32, symbol);
                    w.refresh();
                }
            }
        }
    }
}

fn part2(program: &[Word]) -> Result<(), CpuFault> {
    fn run(
        program: &[Word],
        disp: &mut DisplayCommandInterpreter,
        state: &Arc<Mutex<GameState>>,
    ) -> Result<Word, CpuFault> {
        let mut get_input = || -> Result<Word, InputOutputError> {
            let mut state = state.lock().unwrap();
            let score = format!("{:>10}", state.score);
            let (joystick_pos, indicator) = match state.bat.cmp(&state.ball) {
                Ordering::Less => {
                    // move joystick right
                    (Word(1), ">")
                }
                Ordering::Equal => {
                    // neutral
                    (Word(0), "^")
                }
                Ordering::Greater => {
                    // move joystick left
                    (Word(-1), "<")
                }
            };
            if let Some(w) = state.window.as_mut() {
                const INFO_ROW: i32 = 26;
                w.mvprintw(INFO_ROW, 0, indicator);
                w.mvprintw(INFO_ROW, 20, score);
            }
            //thread::sleep(time::Duration::from_millis(100));
            Ok(joystick_pos)
        };
        let mut do_output = |w: Word| -> Result<(), InputOutputError> {
            match state.lock() {
                Ok(mut state) => {
                    state.update_from(disp.put(w));
                }
                Err(e) => {
                    panic!("lock poisoned: {}", e);
                }
            }
            Ok(())
        };
        let mut cpu = Processor::new(Word(0));
        cpu.load(Word(0), program)?;
        //println!("Memory before inserting coin:\n{:?}", &cpu.ram());
        cpu.load(Word(0), &[Word(2)])?; // insert coin.
                                        //println!("Memory after inserting coin:\n{:?}", &cpu.ram());

        const TRACE_FILE_NAME: &str = "/tmp/aoc-2019-day13-part2-trace-Rust.txt";
        match OpenOptions::new()
            .create(true)
            .write(true)
            .open(TRACE_FILE_NAME)
        {
            Ok(file) => {
                cpu.enable_tracing(file);
            }
            Err(e) => {
                return Err(CpuFault::TraceError(format!(
                    "failed to open trace file {} for writing: {}",
                    TRACE_FILE_NAME, e
                )));
            }
        }
        cpu.run_with_io(&mut get_input, &mut do_output)?;
        Ok(state.lock().unwrap().score)
    }

    let state: Arc<Mutex<GameState>> = Arc::new(Mutex::new(GameState::new()));
    state.lock().unwrap().init();
    let mut disp_interp = DisplayCommandInterpreter::new();
    let result = run(program, &mut disp_interp, &state);
    state.lock().unwrap().done();
    match result {
        Ok(score) => {
            println!("Day 13 part 2: score is {}", score);
            Ok(())
        }
        Err(e) => {
            eprintln!("part2: cpu fault: {}", e);
            Err(e)
        }
    }
}

fn main() -> Result<(), Fail> {
    fn run(words: Vec<Word>) -> Result<(), Fail> {
        part1(&words)?;
        part2(&words)?;
        Ok(())
    }

    run_with_input(13, read_program_from_file, run)
}
