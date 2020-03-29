use std::process;

use intcode;

fn main() {
    let program = intcode::load_program("day9/input.txt").unwrap_or_else(|err| {
        println!("Could not load input file!\n{:?}", err);
        process::exit(1);
    });

    let outputs_part_1 = intcode::run_computer(&program, &[1]).unwrap_or_else(|e| {
        println!("Computer failed: {}", e);
        process::exit(1);
    });
    let outputs_part_2 = intcode::run_computer(&program, &[2]).unwrap_or_else(|e| {
        println!("Computer failed: {}", e);
        process::exit(1);
    });

    println!("Part 1: {}\nPart 2: {}", outputs_part_1[outputs_part_1.len() - 1], outputs_part_2[0]);
}