use std::process;

use intcode;

fn main() {
    let start_time = std::time::Instant::now();

    let program = intcode::load_program("day5/input.txt").unwrap_or_else(|err| {
        println!("Could not load input file!\n{:?}", err);
        process::exit(1);
    });

    let outputs_part_1 = intcode::run_computer(&program, &[1]).unwrap_or_else(|e| {
        println!("Computer failed: {}", e);
        process::exit(1);
    });
    let outputs_part_2 = intcode::run_computer(&program, &[5]).unwrap_or_else(|e| {
        println!("Computer failed: {}", e);
        process::exit(1);
    });

    println!(
        "Part 1: {}\nPart 2: {}\nTime: {}ms",
        outputs_part_1[outputs_part_1.len() - 1],
        outputs_part_2[0],
        start_time.elapsed().as_millis()
    );
}
