use std::process;
use std::sync::mpsc;
use std::thread;

use intcode;

fn main() {
    let program = intcode::load_program("day7/input.txt").unwrap_or_else(|err| {
        println!("Could not load input file!\n{:?}", err);
        process::exit(1);
    });

    let part_1_max_output = find_max_amplifier_signal(&program, &['0', '1', '2', '3', '4'], false);
    let part_2_max_output = find_max_amplifier_signal(&program, &['5', '6', '7', '8', '9'], true);
     
    println!("Part 1: {}\nPart 2: {}", part_1_max_output, part_2_max_output);
}

fn find_max_amplifier_signal(program: &[i64], valid_phase_settings: &[char], feedback_loop_mode: bool) -> i64 {
    let mut max_output = 0;

    for a in valid_phase_settings.to_vec() {
        let val_1 = a.to_string();

        for b in valid_phase_settings.to_vec() {
            let mut val_2 = val_1.clone();
            if val_2.contains(b) { continue; }
            val_2.push(b);

            for c in valid_phase_settings.to_vec() {
                let mut val_3 = val_2.clone();
                if val_3.contains(c) { continue; }
                val_3.push(c);

                for d in valid_phase_settings.to_vec() {
                    let mut val_4 = val_3.clone();
                    if val_4.contains(d) { continue; }
                    val_4.push(d);

                    for e in valid_phase_settings.to_vec() {
                        let mut val_5 = val_4.clone();
                        if val_5.contains(e) { continue; }
                        val_5.push(e);

                        let sequence = val_5.chars().map(|c| i64::from(c.to_digit(10).unwrap())).collect::<Vec<i64>>();
                        let output = if feedback_loop_mode {
                            run_amplifier_feedback_sequence(program, &sequence)
                        } else {
                            run_amplifier_non_feedback_sequence(program, &sequence)
                        };
                        if output > max_output { max_output = output; }
                    }
                }
            }
        }
    }

    max_output
}

fn run_amplifier_non_feedback_sequence(program: &[i64], sequence: &[i64]) -> i64 {
    let mut input = 0;
    for phase_setting in sequence {
        let inputs = vec![*phase_setting, input];
        let outputs = intcode::run_computer(program, &inputs).unwrap_or_else(|e| {
            println!("Computer failed: {}", e);
            process::exit(1);
        });
        input = outputs[0];
    }
    input
}

fn run_amplifier_feedback_sequence(program: &[i64], sequence: &[i64]) -> i64 {
    // The loop is actually going to give the phase settings to amplifiers in the order BCDEA, so
    // take A's setting off the front and put it on the end.
    let mut sequence = sequence.to_vec();
    let a_setting = sequence.remove(0);    
    sequence.push(a_setting);

    let (first_in_send, mut link_recv) = mpsc::channel();

    for phase_setting in sequence {
        let (new_send, new_recv) = mpsc::channel();
        new_send.send(phase_setting).unwrap();
        let mut computer = intcode::Computer::new(program, link_recv, new_send);
        thread::spawn(move || {
            computer.run().unwrap_or_else(|e| {
                println!("Computer failed: {}", e);
                process::exit(1);
            });
        });
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
        if first_in_send.send(output).is_err() { break output; }
    }
}