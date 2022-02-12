use cpu::Processor;
use cpu::Word;
use cpu::{read_program_from_stdin, CpuFault, InputOutputError};
use std::collections::HashSet;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
struct Position {
    x: Word,
    y: Word,
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
        self.pending.push(item);
        match self.pending.as_slice() {
            [Word(-1), Word(0), score] => Some(DrawCommand::UpdateScore(*score)),
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
    println!("Day 13 part 1: {}", block_count);
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
