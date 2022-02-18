use lib::error::Fail;
use lib::input::{read_file_as_string, run_with_input};

const BASE_PATTERN: [i32; 4] = [0, 1, 0, -1];

fn get_pattern(input_len: usize, out_pos: usize) -> Vec<i32> {
    assert!(out_pos > 0); // counted from 1.
    let mut result: Vec<i32> = Vec::with_capacity(input_len + 1);
    let mut pat_pos: usize = 0;
    loop {
        for _repeat in 1..(out_pos + 1) {
            if result.len() > input_len {
                return result.into_iter().skip(1).collect();
            }
            result.push(BASE_PATTERN[pat_pos]);
        }
        pat_pos = (pat_pos + 1) % BASE_PATTERN.len();
    }
}

#[test]
fn test_pattern() {
    fn v(input_len: usize, out_pos: usize) -> Vec<i32> {
        get_pattern(input_len, out_pos)
    }

    assert_eq!(v(10, 1), vec![1, 0, -1, 0, 1, 0, -1, 0, 1, 0]);
    assert_eq!(
        v(15, 2),
        vec![0, 1, 1, 0, 0, -1, -1, 0, 0, 1, 1, 0, 0, -1, -1]
    );
    assert_eq!(v(10, 3), vec![0, 0, 1, 1, 1, 0, 0, 0, -1, -1]);
}

fn fft_digit(input: &[i32], out_pos: usize) -> i32 {
    let pattern = get_pattern(input.len(), out_pos + 1);
    assert_eq!(input.len(), pattern.len());
    let pairs: Vec<(i32, i32)> = input.iter().copied().zip(pattern.into_iter()).collect();
    let total: i32 = pairs.iter().map(|(p, i)| -> i32 { *p * *i }).sum();
    total.abs() % 10
}

fn fft(input: &[i32]) -> Vec<i32> {
    (0..(input.len()))
        .map(|pos| fft_digit(input, pos))
        .collect()
}

fn fft_rounds(input: &[i32], rounds: usize) -> Vec<i32> {
    let mut output = input.to_owned();
    for _round in 0..rounds {
        output = fft(&output);
    }
    output
}

#[test]
fn test_fft() {
    assert_eq!(fft(&[1, 2, 3, 4, 5, 6, 7, 8]), vec![4, 8, 2, 2, 6, 1, 5, 8]);
}

#[test]
fn test_fft_rounds() {
    assert_eq!(
        fft_rounds(&[1, 2, 3, 4, 5, 6, 7, 8], 1),
        vec![4, 8, 2, 2, 6, 1, 5, 8]
    );
    assert_eq!(
        fft_rounds(&[1, 2, 3, 4, 5, 6, 7, 8], 2),
        vec![3, 4, 0, 4, 0, 4, 3, 8]
    );
    assert_eq!(
        fft_rounds(&[1, 2, 3, 4, 5, 6, 7, 8], 3),
        vec![0, 3, 4, 1, 5, 5, 1, 8]
    );
    assert_eq!(
        fft_rounds(&[1, 2, 3, 4, 5, 6, 7, 8], 4),
        vec![0, 1, 0, 2, 9, 4, 9, 8]
    );
}

fn solve1(digits: &[i32]) -> String {
    let result: Vec<String> = fft_rounds(digits, 100)
        .into_iter()
        .take(8)
        .map(|d| d.to_string())
        .collect();
    result.join("")
}

fn part1(digits: &[i32]) -> Result<(), Fail> {
    let v = digits.to_vec();
    println!("Day 16 part 1: {}", solve1(&v));
    Ok(())
}

fn runner(input: String) -> Result<(), Fail> {
    const DECIMAL: u32 = 10;
    let digits: Vec<i32> = input
        .trim()
        .chars()
        .map(|ch: char| -> Result<i32, Fail> {
            match ch.to_digit(DECIMAL) {
                Some(d) => match i32::try_from(d) {
                    Ok(d) => Ok(d),
                    Err(e) => Err(Fail(format!("failed to convert {} to i32: {}", d, e))),
                },
                None => Err(Fail(format!("{} is not a decimal digit", ch))),
            }
        })
        .map(|x| x.expect("todo"))
        .collect();
    part1(&digits)
}

fn main() -> Result<(), Fail> {
    run_with_input(16, read_file_as_string, runner)
}
