use cpu::Processor;
use cpu::Word;
use cpu::{read_program_from_stdin, CpuFault, InputOutputError};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

trait Screen {
    fn draw(&mut self, x: Word, y: Word, tile: Word);
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
struct Position {
    x: Word,
    y: Word,
}

#[derive(Debug)]
struct RecordingScreen {
    tiles: HashMap<Word, HashSet<Position>>,
}

impl RecordingScreen {
    fn new() -> RecordingScreen {
        RecordingScreen {
            tiles: HashMap::new(),
        }
    }

    fn count_blocks(&self) -> usize {
        self.tiles.get(&Word(2)).map(|m| m.len()).unwrap_or(0)
    }
}

impl Screen for RecordingScreen {
    fn draw(&mut self, x: Word, y: Word, tile: Word) {
        self.tiles
            .entry(tile)
            .or_default()
            .insert(Position { x, y });
    }
}

struct DisplayCommandInterpreter {
    recorder: RecordingScreen,
    pending: Vec<Word>,
}

impl DisplayCommandInterpreter {
    fn new(recorder: RecordingScreen) -> DisplayCommandInterpreter {
        DisplayCommandInterpreter {
            recorder,
            pending: Vec::with_capacity(3),
        }
    }

    fn count_blocks(&self) -> usize {
        self.recorder.count_blocks()
    }

    fn put(&mut self, item: Word) {
        self.pending.push(item);
        if let [x, y, block] = self.pending.as_slice() {
            self.recorder.draw(*x, *y, *block);
            self.pending.clear();
        }
    }
}

fn part1(program: &[Word]) -> Result<(), CpuFault> {
    fn run(program: &[Word], disp: &mut DisplayCommandInterpreter) -> Result<(), CpuFault> {
        let mut get_input = || Ok(Word(0));
        let mut do_output = |w: Word| -> Result<(), InputOutputError> {
            disp.put(w);
            Ok(())
        };
        let mut cpu = Processor::new(Word(0));
        cpu.load(Word(0), program)?;
        cpu.run_with_io(&mut get_input, &mut do_output)?;
        Ok(())
    }

    let mut disp_interp = DisplayCommandInterpreter::new(RecordingScreen::new());
    if let Err(e) = run(program, &mut disp_interp) {
        return Err(e);
    }
    println!("Day 13 part 1: {}", disp_interp.count_blocks());
    Ok(())
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
