fn main() {
    let mut count = 0;
    for password in 206938..679129 {
        if evaluate_password(password) {
            println!("{}", password);
            count += 1;
        }
    }
    println!("{}", count);
}

fn evaluate_password(password: u32) -> bool {
    let pass_str = password.to_string();
    let mut chars = pass_str.chars();
    let mut previous_digit = chars.next().unwrap();
    let mut double_digit = false;
    let mut digit_repetition = 1;
    for digit in chars {
        if digit < previous_digit { return false; }
        if digit == previous_digit { 
            digit_repetition += 1;
        } else {
            if digit_repetition == 2 { double_digit = true; }
            digit_repetition = 1;
        }
        previous_digit = digit;
    }
    if digit_repetition == 2 { double_digit = true; }
    double_digit
}