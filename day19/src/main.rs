use std::process;
use std::sync::mpsc::channel;

use intcode;
//#[macro_use] extern crate itertools;

fn main() {
    let memory = intcode::load_program("day19/input.txt").unwrap_or_else(|err| {
        println!("Could not load input file!\n{:?}", err);
        process::exit(1);
    });

    //let mut grid: [[bool; 50]; 50] = [[false; 50]; 50];

    let mut y = 16;
    let mut guess_first_x = 16;
    let mut guess_last_x = 18;
    let mut guessing_first = true;
    loop {        
        let (in_send, in_recv) = channel();
        let (out_send, out_recv) = channel();

        if guessing_first {
            in_send.send(guess_first_x).unwrap();
        } else {
            in_send.send(guess_last_x).unwrap();
        }
        in_send.send(y).unwrap();

        let mut computer = intcode::Computer::new(&memory, in_recv, out_send);
        computer.run().unwrap_or_else(|e| {
            println!("Computer failed: {}", e);
            process::exit(1);
        });
    
        let output = out_recv.recv().unwrap();
        if guessing_first {
            match output {
                0 => guess_first_x += 1,
                1 => guessing_first = false,
                _ => (),
            }
        } else {
            match output {
                1 => guess_last_x += 1,
                0 => {
                    println!("Row {}: {}=>{}", y, guess_first_x, guess_last_x - 1);
                    guess_first_x += 1;
                    y += 1;
                    guessing_first = true;
                },
                _ => (),
            }
        }
        
        if guess_last_x - guess_first_x > 250 { break; }
    }
}
