use std::io;
use std::io::prelude::*;

use cpu::InputOutputError;
use cpu::Processor;
use cpu::Word;

fn run_program(program: &[Word], input_word: Word) -> Vec<Word> {
    let mut cpu = Processor::new(Word(0));
    cpu.load(Word(0), program)
        .expect("should be able to load the program");
    let mut output_words = Vec::new();
    let mut output = |w: Word| -> Result<(), InputOutputError> {
        output_words.push(w);
        Ok(())
    };
    let input: Vec<Word> = vec![input_word];
    if let Err(e) = cpu.run_with_fixed_input(&input, &mut output) {
        panic!("program should be valid: {:?}", e);
    }
    output_words
}

fn part1(program: &[Word]) {
    print!("Day 2 part 1:");
    for w in run_program(program, Word(1)) {
        print!(" {}", w);
    }
    println!();
}

fn part2(program: &[Word]) {
    print!("Day 2 part 2:");
    for w in run_program(program, Word(5)) {
        print!(" {}", w);
    }
    println!();
}

fn main() {
    let words: Vec<Word> = io::BufReader::new(io::stdin())
        .lines()
        .map(|line| line.expect("should be able to read the program"))
        .flat_map(|s| {
            let mut numbers: Vec<Word> = Vec::new();
            for field in s.split(',') {
                match field.parse::<i64>() {
                    Ok(n) => {
                        numbers.push(Word(n));
                    }
                    Err(e) => {
                        panic!("invalid instruction {}: {}", field, e);
                    }
                }
            }
            numbers
        })
        .collect();
    part1(&words);
    part2(&words);
}
