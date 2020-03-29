const MINIMUM_VALUE: u32 = 206_938;
const MAXIMUM_VALUE: u32 = 679_128;

fn main() {
    let start_time = std::time::Instant::now();

    let mut part_1_count = 0;
    let mut part_2_count = 0;
    for password in MINIMUM_VALUE..=MAXIMUM_VALUE {
        if evaluate_password(password, true) {
            part_1_count += 1;
        }
        if evaluate_password(password, false) {
            part_2_count += 1;
        }
    }
    println!(
        "Part 1: {}\nPart 2: {}\nTime: {}ms",
        part_1_count,
        part_2_count,
        start_time.elapsed().as_millis()
    );
}

fn evaluate_password(password: u32, allow_larger_groups: bool) -> bool {
    let pass_str = password.to_string();
    let mut chars = pass_str.chars();
    let mut previous_digit = chars.next().unwrap();
    let mut double_digit = false;
    let mut digit_repetition = 1;
    for digit in chars {
        if digit < previous_digit {
            return false;
        }
        if digit == previous_digit {
            digit_repetition += 1;
        } else {
            if digit_repetition == 2 || (allow_larger_groups && digit_repetition > 2) {
                double_digit = true;
            }
            digit_repetition = 1;
        }
        previous_digit = digit;
    }
    if digit_repetition == 2 || (allow_larger_groups && digit_repetition > 2) {
        double_digit = true;
    }
    double_digit
}
