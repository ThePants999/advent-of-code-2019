use std::process;

use intcode;

fn main() {
    let outputs = intcode::load_and_run_computer("day9/input.txt", &[2]).unwrap_or_else(|e| {
        println!("Computer failed: {}", e);
        process::exit(1);
    });

    println!("{:?}", outputs);
}