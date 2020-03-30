const MINIMUM_VALUE: u32 = 206_938;
const MAXIMUM_VALUE: u32 = 679_128;

// I'm not _unhappy_ with this implementation, but it's simplistic.
// You really want to look at AxlLind's:
// https://github.com/AxlLind/AdventOfCode2019/blob/master/src/bin/04.rs
// It's a thing of beauty, and runs in a fraction of the time of this code.
// I'm not going to work on this further as I'd just be copying his.

fn main() {
    let start_time = std::time::Instant::now();

    let (part_1_valid, part_2_valid) = (MINIMUM_VALUE..=MAXIMUM_VALUE).map(evaluate_password).unzip::<bool, bool, Vec<bool>, Vec<bool>>();
    let part_1_count = part_1_valid.iter().filter(|item| **item).count();
    let part_2_count = part_2_valid.iter().filter(|item| **item).count();

    println!(
        "Part 1: {}\nPart 2: {}\nTime: {}ms",
        part_1_count,
        part_2_count,
        start_time.elapsed().as_millis()
    );
}

// Returns two bools - the first is whether the password is valid by part 1 rules,
// and the second by part 2 rules.
fn evaluate_password(password: u32) -> (bool, bool) {
    let pass_str = password.to_string();
    let mut chars = pass_str.chars();
    let mut previous_digit = chars.next().unwrap();
    let mut at_least_double_digit = false;
    let mut double_digit = false;
    let mut digit_repetition = 1; // How many times have we seen the current digit consecutively?
    for digit in chars {
        if digit < previous_digit {
            return (false, false); // Digits may never decrease
        }
        if digit == previous_digit {
            digit_repetition += 1;
        } else {
            if digit_repetition >= 2 {
                at_least_double_digit = true;
            }
            if digit_repetition == 2 {
                double_digit = true;
            }
            digit_repetition = 1;
        }
        previous_digit = digit;
    }
    if digit_repetition >= 2 {
        at_least_double_digit = true;
    }
    if digit_repetition == 2 {
        double_digit = true;
    }

    (at_least_double_digit, double_digit)
}
