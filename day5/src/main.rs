use std::process;

use intcode;

fn main() {
    let outputs = intcode::load_and_run_computer("day5/input.txt", &vec![5]).unwrap_or_else(|e| {
        println!("Computer failed: {}", e);
        process::exit(1);
    });

    println!("{:?}", outputs);
}