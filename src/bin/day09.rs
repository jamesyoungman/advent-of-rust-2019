use lib::cpu::Processor;
use lib::cpu::Word;
use lib::cpu::{read_program_from_stdin, InputOutputError};

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
    let mut output = run_program(program, Word(1)); // 1 is test mode.
    if let Some(boost_keycode) = output.pop() {
        println!("Day 9 part 1: BOOST keycode is {}", boost_keycode);
    }
    for w in output {
        println!("BOOST self-check thinks opcode {} is not working", &w.0);
    }
}

fn part2(program: &[Word]) {
    let mut output = run_program(program, Word(2)); // 2 is sensor boost mode.
    if let Some(coordinates) = output.pop() {
        println!(
            "Day 9 part 2: Ceres distress signal coordinates {}",
            coordinates
        );
    }
    assert!(output.is_empty());
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
