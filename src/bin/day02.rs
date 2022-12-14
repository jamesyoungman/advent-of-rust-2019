use lib::cpu::{read_program_from_file, InputOutputError, Processor};
use lib::input::run_with_input;
use lib::{cpu::Word, error::Fail};

fn run_program(program: &[Word], noun: Word, verb: Word) -> Word {
    let mut modified_program: Vec<Word> = program.to_vec();
    modified_program[1] = noun;
    modified_program[2] = verb;
    let mut cpu = Processor::new(Word(0));
    cpu.load(Word(0), &modified_program)
        .expect("load base address should be valid");
    let mut discard_output = |_| -> Result<(), InputOutputError> { Ok(()) };
    let no_input = Vec::new();
    if let Err(e) = cpu.run_with_fixed_input(&no_input, &mut discard_output) {
        panic!("program should be valid: {:?}", e);
    }
    let ram = cpu.ram();
    ram[0]
}

fn part1(program: &[Word]) -> Result<(), Fail> {
    println!(
        "Day 2 part 1: location 0 contains {}",
        run_program(program, Word(12), Word(2))
    );
    Ok(())
}

fn part2(program: &[Word]) -> Result<(), Fail> {
    const WANTED: Word = Word(19690720);
    for noun in 1..100 {
        for verb in 1..100 {
            let result: Word = run_program(program, Word(noun), Word(verb));
            if result == WANTED {
                let input = 100 * noun + verb;
                println!("Day 2 part 2: input is {}", input);
                return Ok(());
            }
        }
    }
    Err(Fail("Day 2 part 2: no solution found".to_string()))
}

fn run(words: Vec<Word>) -> Result<(), Fail> {
    part1(&words)?;
    part2(&words)?;
    Ok(())
}

fn main() -> Result<(), Fail> {
    run_with_input(2, read_program_from_file, run)
}
