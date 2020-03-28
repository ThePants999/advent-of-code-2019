use std::process;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::mpsc::channel;
use std::thread;

use intcode;
#[macro_use] extern crate itertools;
extern crate rand;
use rand::{thread_rng, Rng};

fn main() {
    let memory = intcode::load_program("day15/input.txt").unwrap_or_else(|err| {
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

    let mut ship = Ship {
        map: HashMap::new(),
        oxy: HashSet::new(),
        min_x: 0,
        max_x: 0,
        min_y: 0,
        max_y: 0,
    };
    let mut droid_x: i64 = 0;
    let mut droid_y: i64 = 0;
    ship.map.insert(String::from("(0, 0)"), '.');
    let mut moves_since_new_info = 0;

    loop {
        let input = choose_direction(&ship, droid_x, droid_y);
        let (target_x, target_y) = match input {
            1 => (droid_x, droid_y - 1),
            2 => (droid_x, droid_y + 1),
            3 => (droid_x - 1, droid_y),
            4 => (droid_x + 1, droid_y),
            _ => panic!("RNG not working!"),
        };
        in_send.send(input).unwrap();
        let val = match out_recv.recv().unwrap() {
            0 => '#',
            1 => {
                droid_x = target_x;
                droid_y = target_y;
                '.'
            },
            2 => {
                droid_x = target_x;
                droid_y = target_y;
                ship.oxy.insert((target_x, target_y));
                'O'
            },
            _ => panic!("Invalid output from computer"),
        };

        if update_map(&mut ship, target_x, target_y, val) {
            moves_since_new_info = 0;
        } else {
            moves_since_new_info += 1;
            if moves_since_new_info == 100_000 { break; }
        }
    }

    print_map(&ship);

    let mut minutes = 0;
    loop {
        minutes += 1;
        let current_oxy = ship.oxy.clone();
        for (x, y) in current_oxy {
            if !spread_oxygen(&mut ship, x, y) {
                ship.oxy.remove(&(x, y));
            }
        }

        if ship.map.values().filter(|val| **val == '.').count() == 0 {
            break;
        }

        //print_map(&ship);
    }
    
    println!("Minutes: {}", minutes);
}

fn spread_oxygen(ship: &mut Ship, x: i64, y: i64) -> bool {
    try_spread_oxygen(ship, x, y - 1) |
    try_spread_oxygen(ship, x, y + 1) |
    try_spread_oxygen(ship, x - 1, y) |
    try_spread_oxygen(ship, x + 1, y)
}

fn try_spread_oxygen(ship: &mut Ship, x: i64, y: i64) -> bool {
    match consider_target(ship, x, y, true) {
        TargetStates::Visited => {
            ship.oxy.insert((x, y));
            update_map(ship, x, y, 'O');
            true
        },
        _ => false,
    }
}

fn choose_direction(ship: &Ship, droid_x: i64, droid_y: i64) -> i64 {
    let mut non_visited_dirs = Vec::new();
    let mut visited_dirs = Vec::new();
    match consider_target(ship, droid_x, droid_y - 1, false) {
        TargetStates::NotVisited => non_visited_dirs.push(1),
        TargetStates::Visited => visited_dirs.push(1),
        TargetStates::Blocked => (),
    };
    match consider_target(ship, droid_x, droid_y + 1, false) {
        TargetStates::NotVisited => non_visited_dirs.push(2),
        TargetStates::Visited => visited_dirs.push(2),
        TargetStates::Blocked => (),
    };
    match consider_target(ship, droid_x - 1, droid_y, false) {
        TargetStates::NotVisited => non_visited_dirs.push(3),
        TargetStates::Visited => visited_dirs.push(3),
        TargetStates::Blocked => (),
    };
    match consider_target(ship, droid_x + 1, droid_y, false) {
        TargetStates::NotVisited => non_visited_dirs.push(4),
        TargetStates::Visited => visited_dirs.push(4),
        TargetStates::Blocked => (),
    };
    if non_visited_dirs.is_empty() {
        visited_dirs[thread_rng().gen_range(0, visited_dirs.len())]
    } else {
        non_visited_dirs[thread_rng().gen_range(0, non_visited_dirs.len())]
    }
}

enum TargetStates {
    NotVisited,
    Visited,
    Blocked,
}

fn consider_target(ship: &Ship, x: i64, y: i64, oxygen: bool) -> TargetStates {
    let coordinates = format!("({}, {})", x, y);
    match ship.map.get(&coordinates) {
        Some('#') => TargetStates::Blocked,
        Some('O') if oxygen => TargetStates::Blocked,
        None => TargetStates::NotVisited,
        _ => TargetStates::Visited,
    }
}

// Returns true if this was new information.
fn update_map(ship: &mut Ship, x: i64, y: i64, val: char) -> bool {
    let coordinates = format!("({}, {})", x, y);
    if x > ship.max_x { ship.max_x = x; }
    if x < ship.min_x { ship.min_x = x; }
    if y > ship.max_y { ship.max_y = y; }
    if y < ship.min_y { ship.min_y = y; }
    ship.map.insert(coordinates, val).is_none()
}

struct Ship {
    map: HashMap<String, char>,
    oxy: HashSet<(i64, i64)>,
    min_x: i64,
    max_x: i64,
    min_y: i64,
    max_y: i64,
}

fn print_map(ship: &Ship) { //, droid_x: i64, droid_y: i64) {
    let mut drawing = String::new();
    for (y, x) in iproduct!((ship.min_y..=ship.max_y), (ship.min_x..=ship.max_x)) {
        if x == ship.min_x { drawing.push('\n'); }
        //if (x == droid_x) && (y == droid_y) { drawing.push('D'); continue; }
        //if (x == 0) && (y == 0) { drawing.push('O'); continue; }
        let coordinates = format!("({}, {})", x, y);
        match ship.map.get(&coordinates) {
            Some(c) => drawing.push(*c),
            None => drawing.push(' '),
        }
    }

    //let mut file = File::create("day15/output.txt").unwrap();
    //file.write_all(drawing.as_bytes()).unwrap();
    println!("{}", drawing);
}