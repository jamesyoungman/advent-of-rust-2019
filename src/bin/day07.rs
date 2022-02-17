use itertools::Itertools;

use lib::cpu::Word;
use lib::cpu::{read_program_from_stdin, InputOutputError};
use lib::cpu::{CpuFault, CpuStatus, Processor};

fn run_amplifier_chain(program: &[Word], phases: &[Word], input: Word) -> Result<Word, CpuFault> {
    fn run_amplifier(program: &[Word], phase: Word, input: Word) -> Result<Word, CpuFault> {
        let mut cpu = Processor::new(Word(0));
        cpu.load(Word(0), program)?;
        let mut output_words = Vec::new();
        let mut output = |w: Word| -> Result<(), InputOutputError> {
            output_words.push(w);
            Ok(())
        };
        let input = vec![phase, input];
        cpu.run_with_fixed_input(&input, &mut output)?;
        assert_eq!(output_words.len(), 1);
        Ok(output_words[0])
    }

    assert!(!phases.is_empty());
    let (head, tail) = phases.split_at(1);
    let phase = head[0];
    let output: Word = run_amplifier(program, phase, input)?;
    match phases.len() {
        0 => unreachable!(),
        1 => Ok(output),
        _ => run_amplifier_chain(program, tail, output),
    }
}

fn solve1(program: &[Word], input: Word) -> Result<(Word, Vec<Word>), CpuFault> {
    let mut best_output: Option<Word> = None;
    let mut best_phases: Option<Vec<Word>> = None;
    const MAX_PHASE: i64 = 4;
    for phase_permutation in (0..=MAX_PHASE)
        .map(Word)
        .permutations((MAX_PHASE + 1) as usize)
    {
        let output = run_amplifier_chain(program, &phase_permutation, input)?;
        if best_output.unwrap_or(output) <= output {
            best_output = Some(output);
            best_phases = Some(phase_permutation);
        }
    }
    match (best_output, best_phases) {
        (Some(best), Some(phases)) => Ok((best, phases)),
        _ => unreachable!(),
    }
}

#[cfg(test)]
fn check_amplifier_program(
    program: &[i64],
    solver: fn(&[Word], Word) -> Result<(Word, Vec<Word>), CpuFault>,
    expected_best_output: i64,
    expected_best_phases: &[i64],
) {
    fn words(input: &[i64]) -> Vec<Word> {
        input.iter().map(|n| Word(*n)).collect()
    }
    let program = words(program);
    let expected_best_output = Word(expected_best_output);
    let expected_best_phases = words(expected_best_phases);
    match solver(&program, Word(0)) {
        Ok((got_best_output, got_best_phases)) => {
            assert_eq!(
                expected_best_output, got_best_output,
                "incorrect best output"
            );
            assert_eq!(
                expected_best_phases, got_best_phases,
                "incorrect best phases"
            );
        }
        Err(e) => {
            panic!("check_amplifier_program: cpu fault: {}", e);
        }
    }
}

#[cfg(test)]
fn check_amplifier_chain_program(
    program: &[i64],
    expected_best_output: i64,
    expected_best_phases: &[i64],
) {
    check_amplifier_program(program, solve1, expected_best_output, expected_best_phases)
}

#[test]
fn test_amplifier_chain_program() {
    check_amplifier_chain_program(
        &[
            3, 15, 3, 16, 1002, 16, 10, 16, 1, 16, 15, 15, 4, 15, 99, 0, 0,
        ],
        43210,
        &[4, 3, 2, 1, 0],
    );
    check_amplifier_chain_program(
        &[
            3, 23, 3, 24, 1002, 24, 10, 24, 1002, 23, -1, 23, 101, 5, 23, 23, 1, 24, 23, 23, 4, 23,
            99, 0, 0,
        ],
        54321,
        &[0, 1, 2, 3, 4],
    );
    check_amplifier_chain_program(
        &[
            3, 31, 3, 32, 1002, 32, 10, 32, 1001, 31, -2, 31, 1007, 31, 0, 33, 1002, 33, 7, 33, 1,
            33, 31, 31, 1, 32, 31, 31, 4, 31, 99, 0, 0, 0,
        ],
        65210,
        &[1, 0, 4, 3, 2],
    );
}

fn part1(program: &[Word]) {
    match solve1(program, Word(0)) {
        Ok((output, _phases)) => {
            println!("Day 7 part 1: highest output is {}", output);
        }
        Err(e) => {
            eprintln!("Day 7 part 1: cpu failure: {}", e);
        }
    }
}

struct Amplifier {
    cpu: Processor,
    running: bool,
}

impl Amplifier {
    fn new(program: &[Word]) -> Result<Amplifier, CpuFault> {
        let mut cpu = Processor::new(Word(0));
        cpu.load(Word(0), program)?;
        Ok(Amplifier { cpu, running: true })
    }

    fn run_until_output(&mut self, input: Word) -> Result<Option<Word>, CpuFault> {
        assert!(self.running);
        let mut the_output: Option<Word> = None;
        let mut do_output = |w: Word| -> Result<(), InputOutputError> {
            the_output = Some(w);
            Ok(())
        };
        let mut the_input: Option<Word> = Some(input);
        let mut do_input = || {
            if let Some(val) = the_input.take() {
                Ok(val)
            } else {
                Err(InputOutputError::NoInput)
            }
        };
        loop {
            match self.cpu.execute_instruction(&mut do_input, &mut do_output) {
                Ok(CpuStatus::Halt) => {
                    self.running = false;
                    return Ok(the_output);
                }
                Ok(CpuStatus::Run) => (),
                Err(CpuFault::IOError(InputOutputError::NoInput)) => {
                    return Ok(the_output);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }
}

fn run_amplifier_loop(
    program: &[Word],
    phases: &[Word],
    first_input: Word,
) -> Result<Word, CpuFault> {
    // Each amplifier's first input is its phase setting.
    let mut total_halted: usize = 0;
    let mut wires: Vec<Option<Word>> = phases.iter().map(|w| Some(*w)).collect();
    let num_wires = wires.len();
    wires[0] = Some(first_input);
    let mut amplifiers: Vec<Amplifier> =
        match phases.iter().map(|_| Amplifier::new(program)).collect() {
            Ok(v) => v,
            Err(e) => {
                return Err(e);
            }
        };
    let num_amplifiers = amplifiers.len();
    let mut maybe_phases: Vec<Option<Word>> = phases.iter().map(|w| Some(*w)).collect();
    loop {
        for (i, amp) in amplifiers
            .iter_mut()
            .enumerate()
            .filter(|(_, amp)| amp.running)
        {
            let mut input: Option<Word> = match maybe_phases[i].take() {
                Some(phase) => Some(phase),
                None => wires[i].take(),
            };
            if let Some(input) = input.take() {
                match amp.run_until_output(input) {
                    Ok(Some(output)) => {
                        let dest = (i + 1) % num_wires;
                        wires[dest] = Some(output);
                    }
                    Ok(None) => (),
                    Err(e) => {
                        return Err(e);
                    }
                }
                if !amp.running {
                    total_halted += 1;
                    if total_halted == num_amplifiers {
                        if let Some(thruster_input) = wires[0].take() {
                            return Ok(thruster_input);
                        } else {
                            panic!("No thruster input is available");
                        }
                    }
                }
            } else {
                eprintln!("running amplifier {} has no input, skipping it", i);
            }
        }
    }
}

fn solve2(program: &[Word], input: Word) -> Result<(Word, Vec<Word>), CpuFault> {
    let mut best_output: Option<Word> = None;
    let mut best_phases: Option<Vec<Word>> = None;
    for phase_permutation in (5..=9).map(Word).permutations(5) {
        let output = run_amplifier_loop(program, &phase_permutation, input)?;
        if best_output.unwrap_or(output) <= output {
            best_output = Some(output);
            best_phases = Some(phase_permutation);
        }
    }
    if let (Some(best), Some(phases)) = (best_output, best_phases) {
        Ok((best, phases))
    } else {
        unreachable!()
    }
}

#[cfg(test)]
fn check_amplifier_loop_program(
    program: &[i64],
    expected_best_output: i64,
    expected_best_phases: &[i64],
) {
    check_amplifier_program(program, solve2, expected_best_output, expected_best_phases)
}

#[test]
fn test_solve2() {
    check_amplifier_loop_program(
        &[
            3, 26, 1001, 26, -4, 26, 3, 27, 1002, 27, 2, 27, 1, 27, 26, 27, 4, 27, 1001, 28, -1,
            28, 1005, 28, 6, 99, 0, 0, 5,
        ],
        139629729,
        &[9, 8, 7, 6, 5],
    );
    check_amplifier_loop_program(
        &[
            3, 52, 1001, 52, -5, 52, 3, 53, 1, 52, 56, 54, 1007, 54, 5, 55, 1005, 55, 26, 1001, 54,
            -5, 54, 1105, 1, 12, 1, 53, 54, 53, 1008, 54, 0, 55, 1001, 55, 1, 55, 2, 53, 55, 53, 4,
            53, 1001, 56, -1, 56, 1005, 56, 6, 99, 0, 0, 0, 0, 10,
        ],
        18216,
        &[9, 7, 8, 5, 6],
    );
}

fn part2(program: &[Word]) {
    match solve2(program, Word(0)) {
        Ok((output, _)) => {
            println!("Day 7 part 2: highest output is {}", output);
        }
        Err(e) => {
            eprintln!("cpu fault: {}", e);
        }
    }
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
