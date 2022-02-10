use cpu::InputOutputError;
use cpu::Processor;
use cpu::Word;

use cpu::read_program_from_stdin;

fn run_program(program: &[Word], noun: Word, verb: Word) -> Word {
    let mut modified_program: Vec<Word> = program.iter().copied().collect();
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

fn part1(program: &[Word]) {
    println!(
        "Day 2 part 1: location 0 contains {}",
        run_program(program, Word(12), Word(2))
    );
}

fn part2(program: &[Word]) {
    const WANTED: Word = Word(19690720);
    for noun in 1..100 {
        for verb in 1..100 {
            let result: Word = run_program(program, Word(noun), Word(verb));
            if result == WANTED {
                let input = 100 * noun + verb;
                println!("Day 2 part 2: input is {}", input);
                return;
            }
        }
    }
    println!("Day 2 part 2: no solution found");
}

fn main() {
    let words: Vec<Word> = read_program_from_stdin().expect("stdin should be readable");
    part1(&words);
    part2(&words);
}
