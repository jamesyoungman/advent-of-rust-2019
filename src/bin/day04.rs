use std::io::{self, Read};

fn ok(pw: &i32) -> bool {
    let num: String = pw.to_string();
    let mut prev_digit_val: Option<char> = None;
    let mut doubled_digits: usize = 0;
    for digit in num.chars() {
        match prev_digit_val {
            Some(prev) if digit < prev => {
                return false;
            }
            Some(prev) if digit == prev => {
                doubled_digits += 1;
            }
            Some(_) => (),
            None => (),
        }
        prev_digit_val = Some(digit)
    }
    doubled_digits >= 1
}

#[test]
fn test_ok() {
    assert!(ok(&111111));
    assert!(!ok(&223450));
    assert!(!ok(&123789));
    assert!(ok(&112345));
    assert!(ok(&122345));
    assert!(ok(&1356799));
}

fn countpw(pwmin: i32, pwmax: i32) -> usize {
    (pwmin..=pwmax).filter(ok).count()
}

fn main() {
    let mut input = String::new();
    match io::stdin().read_to_string(&mut input) {
        Ok(_) => (),
        Err(e) => {
            panic!("failed to read input: {}", e);
        }
    }
    match input.trim().split_once('-') {
        Some((begin, end)) => match (begin.parse(), end.parse()) {
            (Ok(b), Ok(e)) => {
                println!("Day 4 part 1: {}", countpw(b, e));
            }
            (Err(e), _) | (_, Err(e)) => {
                println!("Day 4 part 1: failed to parse input '{}': {}", input, e);
            }
        },
        None => {
            panic!("input has unexpected format: {}", input);
        }
    }
}
