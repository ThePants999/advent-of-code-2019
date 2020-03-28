use std::process;

#[macro_use] extern crate itertools;
use intcode;

const TARGET: i64 = 19_690_720;

fn main() {
    let memory = intcode::load_program("day2/input.txt").unwrap_or_else(|err| {
        println!("Could not load input file!\n{:?}", err);
        process::exit(1);
    });

    for (noun, verb) in iproduct!(0..memory.len(), 0..memory.len()) {
        let mut memory_copy = memory.clone();
        memory_copy[1] = noun as i64;
        memory_copy[2] = verb as i64;
        let _ = intcode::run_computer(&memory_copy, &Vec::new());
        if memory_copy[0] == TARGET {
            let answer = (noun * 100) + verb;
            println!("{}", answer);
            break;
        }
    }
}
