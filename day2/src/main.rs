#[macro_use] extern crate itertools;
use intcode;
use cjp_threadpool::ThreadPool;

const TARGET: i64 = 19_690_720;

fn main() {
    let start_time = std::time::Instant::now();
    
    let mut memory = intcode::load_program("day2/input.txt").unwrap_or_else(|err| {
        println!("Could not load input file!\n{:?}", err);
        std::process::exit(1);
    });

    // Run the program with the tweaks specified in the question. Extract the value from memory address 0.
    memory[1] = 12;
    memory[2] = 2;
    let (tx, rx) = std::sync::mpsc::channel();
    let mut computer = intcode::ChannelIOComputer::new(&memory, rx, tx);
    computer.run();
    println!("Part 1: {}", computer.fetch_address_zero());

    // Part 2: try every possible combination of values, looking for a combination that
    // results in memory address 0 containing TARGET after execution completes.  Just for
    // the lulz, use a thread pool to parallelise the work.
    let pool = ThreadPool::new_with_default_size();
    for (noun, verb) in iproduct!(0..memory.len(), 0..memory.len()) {
        let mut memory_copy = memory.clone();
        memory_copy[1] = noun as i64;
        memory_copy[2] = verb as i64;

        pool.schedule(Box::new(move || {
            let (tx, rx) = std::sync::mpsc::channel();
            let mut computer = intcode::ChannelIOComputer::new(&memory_copy, rx, tx);
            computer.run();
            if computer.fetch_address_zero() == TARGET {
                Some((noun * 100) + verb)
            } else {
                None
            }
        }));
    }

    let answer = loop {
        if let Some(answer) = pool.results.recv().unwrap() { break answer; }
    };
    pool.terminate();
    println!("Part 2: {}\nTime: {}ms", answer, start_time.elapsed().as_millis());
}
