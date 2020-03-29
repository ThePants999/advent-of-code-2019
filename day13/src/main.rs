#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]

use std::collections::HashMap;
use std::process;
use std::sync::mpsc;

use intcode;
#[macro_use]
extern crate itertools;

fn main() {
    let start_time = std::time::Instant::now();

    let mut program = intcode::load_program("day13/input.txt").unwrap_or_else(|err| {
        println!("Could not load input file!\n{:?}", err);
        process::exit(1);
    });

    // Part 1
    let outputs_part_1 = intcode::run_computer(&program, &[]).unwrap_or_else(|e| {
        println!("Computer failed: {}", e);
        process::exit(1);
    });
    let blocks = outputs_part_1
        .iter()
        .skip(2)
        .step_by(3)
        .filter(|tile_id| **tile_id == 2)
        .count();

    // Part 2
    program[0] = 2;
    let (in_send, in_recv) = mpsc::channel();
    let (out_send, out_recv) = mpsc::channel();
    let mut computer = intcode::Computer::new(&program, in_recv, out_send);
    std::thread::spawn(move || {
        computer.run().unwrap_or_else(|e| {
            println!("Computer failed: {}", e);
            process::exit(1);
        });
    });

    let mut screen = Screen {
        width: 1,
        height: 1,
        tiles: HashMap::new(),
        ball_x: 10,
        paddle_x: 10,
        score: 0,
    };

    #[allow(clippy::while_let_loop)]
    loop {
        let output1 = match out_recv.recv() {
            Ok(val) => val,
            Err(_) => break,
        };
        let output2 = match out_recv.recv() {
            Ok(val) => val,
            Err(_) => break,
        };
        let output3 = match out_recv.recv() {
            Ok(val) => val,
            Err(_) => break,
        };

        if (output1 == -1) && (output2 == 0) {
            screen.score = output3;
            continue;
        }

        let x = output1 as usize;
        let y = output2 as usize;
        let tile_id = output3;
        let coordinates = format!("({}, {})", x, y);
        if !screen.tiles.contains_key(&coordinates) {
            if x >= screen.width {
                screen.width = x + 1;
            }
            if y >= screen.height {
                screen.height = y + 1;
            }
        }
        screen.tiles.insert(coordinates, tile_id);

        if tile_id == 3 {
            screen.paddle_x = x;
        } else if tile_id == 4 {
            screen.ball_x = x;
            match x {
                less if less < screen.paddle_x => in_send.send(-1).unwrap(),
                more if more > screen.paddle_x => in_send.send(1).unwrap(),
                _ => in_send.send(0).unwrap(),
            }
        }
    }

    println!(
        "Part 1: {}\nPart 2: {}\nTime: {}ms",
        blocks,
        screen.score,
        start_time.elapsed().as_millis()
    );
}

fn _print_screen(screen: &mut Screen) {
    let mut output = screen.score.to_string();

    for (y, x) in iproduct!((0..screen.height), (0..screen.width)) {
        if x == 0 {
            output.push('\n');
        }
        let coordinates = format!("({}, {})", x, y);
        output.push(match screen.tiles.get(&coordinates) {
            Some(1) => '#',
            Some(2) => '$',
            Some(3) => '_',
            Some(4) => 'o',
            _ => ' ',
        });
    }
    println!("{}", output);
}

struct Screen {
    width: usize,
    height: usize,
    tiles: HashMap<String, i64>,
    ball_x: usize,
    paddle_x: usize,
    score: i64,
}
