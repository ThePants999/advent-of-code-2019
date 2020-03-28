use std::cell::RefCell;
use std::thread;
use std::sync::mpsc;

extern crate num;
extern crate primes;

fn simulate(moons: Vec<RefCell<MoonDimension>>) -> u64 {
    let initial_state = moons.clone();
    let mut steps = 0;
    loop {
        perform_step(&moons);
        steps += 1;
        if moons == initial_state { break steps; }
    }
}

fn perform_step(moons: &Vec<RefCell<MoonDimension>>) {
    for this_moon_cell in moons {
        let mut this_moon = this_moon_cell.borrow_mut();
        for other_moon_cell in moons {
            if this_moon_cell as *const _ == other_moon_cell as *const _ { continue; }
            calc_vel_change(&mut this_moon, &other_moon_cell.borrow());
        }
    }
    for this_moon_cell in moons {
        let mut this_moon = this_moon_cell.borrow_mut();
        apply_vel_change(&mut this_moon);
    }
}

fn calc_vel_change(this_moon: &mut MoonDimension, other_moon: &MoonDimension) {
    if other_moon.pos > this_moon.pos { this_moon.vel += 1; }
    if other_moon.pos < this_moon.pos { this_moon.vel -= 1; }
}

fn apply_vel_change(moon: &mut MoonDimension) {
    moon.pos += moon.vel;
}

fn main() {
    let dimensions = vec![
        vec![
            RefCell::new(MoonDimension { pos: -10, vel: 0, }),
            RefCell::new(MoonDimension { pos: 5, vel: 0, }),
            RefCell::new(MoonDimension { pos: 3, vel: 0, }),
            RefCell::new(MoonDimension { pos: 1, vel: 0, }),
        ],
        vec![
            RefCell::new(MoonDimension { pos: -10, vel: 0, }),
            RefCell::new(MoonDimension { pos: 5, vel: 0, }),
            RefCell::new(MoonDimension { pos: 8, vel: 0, }),
            RefCell::new(MoonDimension { pos: 3, vel: 0, }),
        ],
        vec![
            RefCell::new(MoonDimension { pos: -13, vel: 0, }),
            RefCell::new(MoonDimension { pos: -9, vel: 0, }),
            RefCell::new(MoonDimension { pos: -16, vel: 0, }),
            RefCell::new(MoonDimension { pos: -3, vel: 0, }),
        ],
    ];

    let mut channels = Vec::new();
    for dimension in dimensions {
        let (tx, rx) = mpsc::channel();
        channels.push(rx);
        thread::spawn(move || {
            tx.send(simulate(dimension)).unwrap();
        });
    }

    let mut steps = Vec::new();
    for rx in channels {
        steps.push(rx.recv().unwrap());
    }

    println!("{}", lowest_common_multiple(steps));
}

fn lowest_common_multiple(nums: Vec<u64>) -> u64 {
    let mut lcm = 1;

    let mut prime_factors = Vec::new();
    let mut unique_prime_factors = Vec::new();
    for num in nums {
        let factors = primes::factors(num);
        unique_prime_factors.append(&mut factors.clone());
        prime_factors.push(factors);
    }
    unique_prime_factors.sort_unstable();
    unique_prime_factors.dedup();

    for unique_factor in unique_prime_factors {
        let mut max_count = 1;
        for factor_set in &prime_factors {
            let mut count = 0;
            for factor in factor_set {
                if *factor == unique_factor { count += 1; }
            }
            if count > max_count { max_count = count; }
        }
        lcm *= num::pow::pow(unique_factor, max_count);
    }

    lcm
}

#[derive(Clone, PartialEq, Eq)]
struct MoonDimension {
    pos: i32,
    vel: i32,
    // pos_x: i32,
    // pos_y: i32,
    // pos_z: i32,
    // vel_x: i32,
    // vel_y: i32,
    // vel_z: i32,
}

// impl Moon {
//     fn potential_energy(&self) -> i32 {
//         self.pos_x.abs() + self.pos_y.abs() + self.pos_z.abs()
//     }

//     fn kinetic_energy(&self) -> i32 {
//         self.vel_x.abs() + self.vel_y.abs() + self.vel_z.abs()
//     }

//     fn total_energy(&self) -> i32 {
//         self.potential_energy() * self.kinetic_energy()
//     }
// }