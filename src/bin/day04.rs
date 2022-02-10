use aoc::read_stdin_as_string;

fn ok(pw: &i32, doubling_limit: usize) -> bool {
    let num: String = pw.to_string();
    let mut prev_digit_val: Option<char> = None;
    let mut double_count: [usize; 10] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    for digit in num.chars() {
        match prev_digit_val {
            Some(prev) if digit < prev => {
                return false;
            }
            Some(prev) if digit == prev => match digit.to_digit(10) {
                Some(d) => {
                    double_count[d as usize] += 1;
                }
                None => {
                    panic!("not a valid digit: {}", digit);
                }
            },
            Some(_) => (),
            None => (),
        }
        prev_digit_val = Some(digit)
    }
    double_count.iter().any(|n| *n >= 1 && *n <= doubling_limit)
}

#[test]
fn test_ok() {
    assert!(ok(&111111, usize::MAX));
    assert!(!ok(&223450, usize::MAX));
    assert!(!ok(&123789, usize::MAX));
    assert!(ok(&112345, usize::MAX));
    assert!(ok(&122345, usize::MAX));
    assert!(ok(&1356799, usize::MAX));

    // part 2
    assert!(!ok(&123444, 1));
    assert!(ok(&11122, 1));
}

fn countpw(pwmin: i32, pwmax: i32, limit: usize) -> usize {
    let is_ok = |pw: &i32| -> bool { ok(pw, limit) };
    (pwmin..=pwmax).filter(is_ok).count()
}

fn main() {
    let input = read_stdin_as_string().expect("should be able to read input");
    match input.trim().split_once('-') {
        Some((begin, end)) => match (begin.parse(), end.parse()) {
            (Ok(b), Ok(e)) => {
                println!("Day 4 part 1: {}", countpw(b, e, usize::MAX));
                println!("Day 4 part 2: {}", countpw(b, e, 1));
            }
            (Err(e), _) | (_, Err(e)) => {
                println!("Day 4: failed to parse input '{}': {}", input, e);
            }
        },
        None => {
            panic!("input has unexpected format: {}", input);
        }
    }
}
