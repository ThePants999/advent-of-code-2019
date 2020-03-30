use std::sync::mpsc;

use intcode;
use itertools::Itertools;

fn main() {
    let start_time = std::time::Instant::now();

    let program = intcode::load_program("day7/input.txt").unwrap_or_else(|err| {
        println!("Could not load input file!\n{:?}", err);
        std::process::exit(1);
    });

    let part_1_max_output = find_max_amplifier_signal(
        &program,
        &[0, 1, 2, 3, 4],
        &run_amplifier_non_feedback_sequence,
    );
    let part_2_max_output = find_max_amplifier_signal(
        &program,
        &[5, 6, 7, 8, 9],
        &run_amplifier_feedback_sequence);

    println!(
        "Part 1: {}\nPart 2: {}\nTime: {}ms",
        part_1_max_output,
        part_2_max_output,
        start_time.elapsed().as_millis()
    );
}

fn find_max_amplifier_signal(
    program: &[i64],
    valid_phase_settings: &[i64],
    sequence_func: &dyn Fn(&[i64], &[i64]) -> i64,
) -> i64 {
    // Run the amplifier sequence for each permutation of the phase settings
    // provided, and find the highest output.
    valid_phase_settings
        .into_iter()
        .copied()
        .permutations(5)
        .map(|sequence| sequence_func(program, &sequence))
        .max()
        .unwrap()
}

fn run_amplifier_non_feedback_sequence(program: &[i64], sequence: &[i64]) -> i64 {
    // Run the computer once for each phase setting in the sequence provided.
    // Give each computer, as inputs, the corresponding phase setting, plus the output
    // from the previous computer. (0 for the first one.)
    let mut input = 0;
    for phase_setting in sequence {
        let inputs = vec![*phase_setting, input];
        let outputs = intcode::run_computer(program, &inputs);
        input = outputs[0];
    }
    input
}

fn run_amplifier_feedback_sequence(program: &[i64], sequence: &[i64]) -> i64 {
    // Run a computer for each phase setting in the sequence provided, but
    // leave them all running indefinitely, each on its own thread. Give each one
    // the previous computer's output channel as its input channel, so they're
    // passing data to one another, except bootstrap each channel with the phase
    // setting.  The link from the last computer back to the first goes via
    // this thread, so we can cache what we see and use the last seen value
    // when the programs terminate.

    // The loop below is actually going to give the phase settings to amplifiers
    // in the order BCDEA, so take A's setting off the front and put it on the end.
    let mut sequence = sequence.to_vec();
    let a_setting = sequence.remove(0);
    sequence.push(a_setting);

    let (first_in_send, mut link_recv) = mpsc::channel();

    for phase_setting in sequence {
        let (new_send, new_recv) = mpsc::channel();
        new_send.send(phase_setting).unwrap();
        let mut computer = intcode::Computer::new(program, link_recv, new_send);
        std::thread::spawn(move || { computer.run(); });
        link_recv = new_recv;
    }

    // The dangling receive we have is the output from amplifier E. We actually put A's phase setting
    // into this channel before we gave the send half to that amplifier, so it's sitting in the
    // receive buffer. We need to kick the process off by sending 0 to A _after_ its phase setting,
    // so pick up the phase setting and push it into A, followed by 0.
    first_in_send.send(link_recv.recv().unwrap()).unwrap();
    first_in_send.send(0).unwrap();

    // The computer's running now - we just need to chain outputs from E to A until we can't any more.
    // The last output is the result.
    let mut output: i64 = 0;
    loop {
        output = link_recv.recv().unwrap_or(output);
        if first_in_send.send(output).is_err() {
            break output;
        }
    }
}
