use std::collections::HashMap;

use lib::cpu::{read_program_from_file, InputOutputError, Processor, Word};
use lib::error::Fail;
use lib::grid::{bounds, Position};
use lib::input::run_with_input;

use ndarray::prelude::*;

struct ImageBuilder {
    pos: Position,
    pixels: HashMap<Position, char>,
}

impl ImageBuilder {
    fn new() -> ImageBuilder {
        ImageBuilder {
            pos: Position { x: 0, y: 0 },
            pixels: HashMap::new(),
        }
    }

    fn emit(&mut self, ch: char) {
        match ch {
            '\n' => {
                self.pos.y += 1;
                self.pos.x = 0;
            }
            _ => {
                self.pixels.insert(self.pos, ch);
                self.pos.x += 1;
            }
        }
    }

    fn getter(&self, r: usize, c: usize) -> char {
        match (c.try_into(), r.try_into()) {
            (Ok(x), Ok(y)) => match self.pixels.get(&Position { x, y }) {
                Some(ch) => *ch,
                None => '?',
            },
            _ => '!',
        }
    }

    fn build(&self) -> Array2<char> {
        match bounds(self.pixels.keys()) {
            Some((min, max)) => {
                let w = max.x - min.x;
                let h = max.y - min.y;
                let shape = (h as usize, w as usize);
                Array2::from_shape_fn(shape, |(r, c)| self.getter(r, c))
            }
            None => Array2::from_shape_fn((0, 0), |(_, _)| '^'),
        }
    }
}

fn is_scaffold(arr: &Array2<char>, pos: &(usize, usize)) -> bool {
    matches!(arr[*pos], '#' | '^' | 'v' | '>' | '<')
}

fn is_scaffold_intersection(arr: &Array2<char>, pos: &(usize, usize)) -> bool {
    let (h, w) = match arr.shape() {
        &[h, w] => (h, w),
        _ => {
            panic!("unexpected shape in array");
        }
    };

    // check centre
    if !is_scaffold(arr, pos) {
        return false;
    }
    if pos.0 > 0 {
        // check north neighbour (note, y axis points down the page)
        if !is_scaffold(arr, &(pos.0 - 1, pos.1)) {
            return false;
        }
    }
    if pos.1 > 0 {
        // check west neighbour
        if !is_scaffold(arr, &(pos.0, pos.1 - 1)) {
            return false;
        }
    }
    if pos.0 < h {
        // check south neighbour
        if !is_scaffold(arr, &(pos.0 + 1, pos.1)) {
            return false;
        }
    }
    if pos.1 < w {
        // check east neighbour
        if !is_scaffold(arr, &(pos.0, pos.1 + 1)) {
            return false;
        }
    }
    true
}

fn find_matches<F>(array: &Array2<char>, pred: F) -> Vec<Position>
where
    F: Fn(&Array2<char>, &(usize, usize)) -> bool,
{
    array
        .indexed_iter()
        .filter(|(pos, _)| pred(array, &(pos.0, pos.1)))
        .map(|(pos, _)| Position {
            y: pos.0 as i64,
            x: pos.1 as i64,
        })
        .collect()
}

fn alignment_parameter(pos: &Position) -> i64 {
    pos.x * pos.y
}

fn part1(program: &[Word]) -> Result<(), Fail> {
    let mut cpu: Processor = Processor::new(Word(0));
    cpu.load(Word(0), program)?;
    let mut imb = ImageBuilder::new();
    let mut get_input = || -> Result<Word, InputOutputError> { Err(InputOutputError::NoInput) };
    let mut do_output = |w: Word| -> Result<(), InputOutputError> {
        if let Ok(Ok(ch)) = u32::try_from(w.0).map(char::try_from) {
            print!("{}", ch);
            imb.emit(ch);
            Ok(())
        } else {
            Err(InputOutputError::Unprintable(w))
        }
    };
    cpu.run_with_io(&mut get_input, &mut do_output)?;
    let array = imb.build();
    let matches = find_matches(&array, is_scaffold_intersection);
    println!("{:?}", &matches);
    let tot: i64 = matches.iter().map(alignment_parameter).sum();
    println!("Day 17 part 1: count is {}, sum is {}", matches.len(), tot);
    Ok(())
}

fn run(words: Vec<Word>) -> Result<(), Fail> {
    part1(&words)
}

fn main() -> Result<(), Fail> {
    run_with_input(17, read_program_from_file, run)
}
