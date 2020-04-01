#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]

use std::sync::mpsc;

use intcode;

// Part 1's logic is straightforward: jump if we have to (there's a hole at 1, 2 or 3) and if
// we can (there isn't one at 4).
const PART_1_SCRIPT: &str = 
"NOT A J
NOT B T
OR T J
NOT C T
OR T J
AND D J
WALK
";

// Part 2's logic bolts onto that: don't jump unless either we can jump again immediately on
// landing (no hole at 8) or we don't need to (no hole at 5).
const PART_2_SCRIPT: &str = 
"NOT A J
NOT B T
OR T J
NOT C T
OR T J
AND D J
NOT J T
OR E T
OR H T
AND T J    
RUN
";

fn main() {
    let start_time = std::time::Instant::now();
    let program = intcode::load_program("day21/input.txt").unwrap_or_else(|err| {
        println!("Could not load input file!\n{:?}", err);
        std::process::exit(1);
    });

    let part_1_result = run_program(&program, PART_1_SCRIPT);
    let part_2_result = run_program(&program, PART_2_SCRIPT);
    println!(
        "Part 1: {}\nPart 2: {}\nTime: {}ms",
        part_1_result,
        part_2_result,
        start_time.elapsed().as_millis(),
    );
}

fn run_program(intcode: &[i64], springscript: &str) -> i64 {
    let (in_send, in_recv) = mpsc::channel();
    let (out_send, out_recv) = mpsc::channel();
    let mut computer = intcode::ChannelIOComputer::new(&intcode, in_recv, out_send);
    std::thread::spawn(move || {
        computer.run();
    });

    for c in springscript.chars() {
        in_send.send(c as i64).unwrap();
    }

    while let Ok(input) = out_recv.recv() {
        if input > 255 {
            return input;
        }
    }
    unreachable!()
}
