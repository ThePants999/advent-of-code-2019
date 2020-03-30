#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]

use std::sync::mpsc::channel;

use intcode;

fn main() {
    let program = intcode::load_program("day25/input.txt").unwrap_or_else(|err| {
        println!("Could not load input file!\n{:?}", err);
        std::process::exit(1);
    });

    let (in_send, in_recv) = channel();
    let (out_send, out_recv) = channel();
    let mut computer = intcode::Computer::new(&program, in_recv, out_send);
    std::thread::spawn(move || { computer.run(); });

    loop {
        let mut display = String::new();
        loop {
            let c = out_recv.recv().unwrap() as u8 as char;
            if c == '\n' {
                println!("{}", display);
                if display.eq("Command?") { break; }
                display.clear();
            } else {
                display.push(c);
            }
        }

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).expect("Did not enter a string");
        if let Some('\n') = input.chars().next_back() {
            input.pop();
        }
        if let Some('\r') = input.chars().next_back() {
            input.pop();
        }
        input.push('\n');
        for c in input.chars() {
            in_send.send(c as i64).unwrap();
        }
    }
}
