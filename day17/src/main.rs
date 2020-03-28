#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]

use std::process;
use std::sync::mpsc::channel;
use std::thread;

use intcode;

fn main() {
    let memory = intcode::load_program("day17/input.txt").unwrap_or_else(|err| {
        println!("Could not load input file!\n{:?}", err);
        process::exit(1);
    });

    let (in_send, in_recv) = channel();
    let (out_send, out_recv) = channel();
    let mut computer = intcode::Computer::new(&memory, in_recv, out_send);
    thread::spawn(move || {
        computer.run().unwrap_or_else(|e| {
            println!("Computer failed: {}", e);
            process::exit(1);
        });
    });

    let program = String::from("B,B,A,C,A,C,A,C,B,C\nR,4,R,6,R,6,R,4,R,4\nR,6,L,8,R,8\nL,8,R,6,L,10,L,10\nn\n");
    for c in program.chars() {
        in_send.send(c as i64).unwrap();
    }

    let mut display = String::new();
    while let Ok(input) = out_recv.recv() {
        if input > 255 {
            println!("Dust: {}", input);
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


    //let dust = out_recv.recv().unwrap();
    //println!("Dust: {}", dust);

    // let mut grid: Vec<Vec<char>> = Vec::new();
    // let mut current_row = Vec::new();
    // while let Ok(input) = out_recv.recv() {
    //     match input as u8 as char {
    //         '\n' => {
    //             if current_row.len() > 0 {
    //                 grid.push(current_row);
    //                 current_row = Vec::new();
    //             }
    //         },
    //         //'^' | '>' | 'v' | '<' => current_row.push('#'),
    //         c => current_row.push(c),
    //     };
    // }

    // let height = grid.len();
    // let width = grid[0].len();
    // let mut output = String::new();
    // let mut alignment = 0;
    // for (y, col) in grid.iter().enumerate() {
    //     for (x, c) in col.iter().enumerate() {
    //         if (*c == '#') &&
    //             (y > 0 && grid[y - 1][x] == '#') &&
    //             (y < (height - 1) && grid[y + 1][x] == '#') &&
    //             (x > 0 && grid[y][x - 1] == '#') &&
    //             (x < (width - 1) && grid[y][x + 1] == '#') {
    //                 output.push('O');
    //                 alignment += x * y;
    //         } else {
    //             output.push(*c);
    //         }
    //     }
    //     output.push('\n');
    // }
    // println!("{}\nAlignment: {}", output, alignment);
}
