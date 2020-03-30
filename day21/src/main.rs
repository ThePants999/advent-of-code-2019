#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]

use std::sync::mpsc;

use intcode;

fn main() {
    let memory = intcode::load_program("day21/input.txt").unwrap_or_else(|err| {
        println!("Could not load input file!\n{:?}", err);
        std::process::exit(1);
    });

    let (in_send, in_recv) = mpsc::channel();
    let (out_send, out_recv) = mpsc::channel();
    let mut computer = intcode::Computer::new(&memory, in_recv, out_send);
    std::thread::spawn(move || { computer.run(); });

    let program = String::from("NOT A T
    OR T J
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
    ");
    for c in program.chars() {
        in_send.send(c as i64).unwrap();
    }

    let mut display = String::new();
    while let Ok(input) = out_recv.recv() {
        if input > 255 {
            println!("Damage: {}", input);
        } else {
            match input as u8 as char {
                '\n' => {
                    println!("{}", display);
                    display = String::new();
                },
                c => display.push(c),
            };    
        }
    }
}
