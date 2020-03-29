use std::process;

#[macro_use] extern crate itertools;
use intcode;

const TARGET: i64 = 19_690_720;

fn main() {
    let memory = intcode::load_program("day2/input.txt").unwrap_or_else(|err| {
        println!("Could not load input file!\n{:?}", err);
        process::exit(1);
    });

    let mut memory_copy = memory.clone();
    memory_copy[1] = 12;
    memory_copy[2] = 2;
    let (tx, rx) = std::sync::mpsc::channel();
    let mut computer = intcode::Computer::new(&memory_copy, rx, tx);
    computer.run().unwrap();
    println!("Part 1: {}", computer.fetch_from_address(0).unwrap());

    for (noun, verb) in iproduct!(0..memory.len(), 0..memory.len()) {
        let mut memory_copy = memory.clone();
        memory_copy[1] = noun as i64;
        memory_copy[2] = verb as i64;
        let (tx, rx) = std::sync::mpsc::channel();
        let mut computer = intcode::Computer::new(&memory_copy, rx, tx);
        computer.run().unwrap();
        if computer.fetch_from_address(0).unwrap() == TARGET {
            let answer = (noun * 100) + verb;
            println!("Part 2: {}", answer);
            break;
        }
    }
}
