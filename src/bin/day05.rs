use lib::cpu::Word;
use lib::cpu::{read_program_from_file, InputOutputError, Processor};
use lib::input::run_with_input;

use lib::error::Fail;

fn run_program(program: &[Word], input_word: Word) -> Result<Vec<Word>, Fail> {
    let mut cpu = Processor::new(Word(0));
    cpu.load(Word(0), program)
        .map_err(|e| Fail(e.to_string()))?;
    let mut output_words = Vec::new();
    let mut output = |w: Word| -> Result<(), InputOutputError> {
        output_words.push(w);
        Ok(())
    };
    let input: Vec<Word> = vec![input_word];
    if let Err(e) = cpu.run_with_fixed_input(&input, &mut output) {
        Err(Fail(format!("program should be valid: {:?}", e)))
    } else {
        Ok(output_words)
    }
}

fn part1(program: &[Word]) -> Result<(), Fail> {
    print!("Day 2 part 1:");
    for w in run_program(program, Word(1))? {
        print!(" {}", w);
    }
    println!();
    Ok(())
}

fn part2(program: &[Word]) -> Result<(), Fail> {
    print!("Day 2 part 2:");
    for w in run_program(program, Word(5))? {
        print!(" {}", w);
    }
    println!();
    Ok(())
}

fn run(words: Vec<Word>) -> Result<(), Fail> {
    part1(&words)?;
    part2(&words)?;
    Ok(())
}

fn main() -> Result<(), Fail> {
    run_with_input(5, read_program_from_file, run)
}
