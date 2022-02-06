use itertools::Itertools;

use cpu::Processor;
use cpu::Word;
use cpu::{read_program_from_stdin, InputOutputError};

fn run_amplifier(program: &[Word], phase: Word, input: Word) -> Word {
    let mut cpu = Processor::new(Word(0));
    cpu.load(Word(0), program)
        .expect("should be able to load the program");
    let mut output_words = Vec::new();
    let mut output = |w: Word| -> Result<(), InputOutputError> {
        output_words.push(w);
        Ok(())
    };
    let input = vec![phase, input];
    if let Err(e) = cpu.run_with_fixed_input(&input, &mut output) {
        panic!("program should be valid: {:?}", e);
    }
    assert_eq!(output_words.len(), 1);
    output_words[0]
}

fn run_chain(program: &[Word], phases: &[Word], input: Word) -> Word {
    assert!(!phases.is_empty());
    let (head, tail) = phases.split_at(1);
    let phase = head[0];
    let output = run_amplifier(program, phase, input);
    match phases.len() {
        0 => unreachable!(),
        1 => output,
        _ => run_chain(program, tail, output),
    }
}

fn solve1(program: &[Word], input: Word) -> (Word, Vec<Word>) {
    let mut best_output: Option<Word> = None;
    let mut best_phases: Option<Vec<Word>> = None;
    const MAX_PHASE: i64 = 4;
    for phase_permutation in (0..=MAX_PHASE)
        .map(Word)
        .permutations((MAX_PHASE + 1) as usize)
    {
        let output = run_chain(program, &phase_permutation, input);
        if best_output.unwrap_or(output) <= output {
            best_output = Some(output);
            best_phases = Some(phase_permutation);
        }
    }
    match (best_output, best_phases) {
        (Some(best), Some(phases)) => (best, phases),
        _ => unreachable!(),
    }
}

#[cfg(test)]
fn check_amplifier_program(
    program: &[i64],
    expected_best_output: i64,
    expected_best_phases: &[i64],
) {
    fn words(input: &[i64]) -> Vec<Word> {
        input.iter().map(|n| Word(*n)).collect()
    }
    let program = words(program);
    let expected_best_output = Word(expected_best_output);
    let expected_best_phases = words(expected_best_phases);
    let (got_best_output, got_best_phases) = solve1(&program, Word(0));
    assert_eq!(
        expected_best_output, got_best_output,
        "incorrect best output"
    );
    assert_eq!(
        expected_best_phases, got_best_phases,
        "incorrect best phases"
    );
}

#[test]
fn test_amplifier_program() {
    check_amplifier_program(
        &[
            3, 15, 3, 16, 1002, 16, 10, 16, 1, 16, 15, 15, 4, 15, 99, 0, 0,
        ],
        43210,
        &[4, 3, 2, 1, 0],
    );
    check_amplifier_program(
        &[
            3, 23, 3, 24, 1002, 24, 10, 24, 1002, 23, -1, 23, 101, 5, 23, 23, 1, 24, 23, 23, 4, 23,
            99, 0, 0,
        ],
        54321,
        &[0, 1, 2, 3, 4],
    );
    check_amplifier_program(
        &[
            3, 31, 3, 32, 1002, 32, 10, 32, 1001, 31, -2, 31, 1007, 31, 0, 33, 1002, 33, 7, 33, 1,
            33, 31, 31, 1, 32, 31, 31, 4, 31, 99, 0, 0, 0,
        ],
        65210,
        &[1, 0, 4, 3, 2],
    );
}

fn part1(program: &[Word]) {
    let (output, _phases) = solve1(program, Word(0));
    println!("Day 7 part 1: highest output is {}", output);
}

fn main() {
    match read_program_from_stdin() {
        Ok(words) => {
            part1(&words);
        }
        Err(e) => {
            eprintln!("failed to load program: {}", e);
        }
    }
}
