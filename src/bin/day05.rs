use lib::cpu::read_program_from_stdin;
use lib::cpu::Word;
use lib::cpu::{InputOutputError, Processor};

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
    match read_program_from_stdin() {
        Ok(words) => {
            part1(&words);
            part2(&words);
        }
        Err(e) => {
            eprintln!("failed to load program: {}", e);
        }
    }
}
