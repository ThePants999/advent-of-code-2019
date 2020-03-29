use std::cell::RefCell;

extern crate num;
extern crate primes;
#[macro_use] extern crate itertools;

fn simulate(moons: &[RefCell<MoonDimension>]) -> u64 {
    let moons = moons.to_owned();
    let initial_state = moons.clone();
    let mut steps = 0;
    loop {
        perform_step(&moons);
        steps += 1;
        if moons == initial_state { break steps; }
    }
}

fn perform_step(moons: &[RefCell<MoonDimension>]) {
    for this_moon_cell in moons {
        for other_moon_cell in moons {
            if this_moon_cell as *const _ == other_moon_cell as *const _ { continue; }
            calc_vel_change(this_moon_cell, other_moon_cell);
        }
    }
    for this_moon_cell in moons {
        apply_vel_change(this_moon_cell);
    }
}

fn calc_vel_change(this_moon_cell: &RefCell<MoonDimension>, other_moon_cell: &RefCell<MoonDimension>) {
    let mut this_moon = this_moon_cell.borrow_mut();
    let other_moon = other_moon_cell.borrow();
    if other_moon.pos > this_moon.pos { this_moon.vel += 1; }
    if other_moon.pos < this_moon.pos { this_moon.vel -= 1; }
}

fn apply_vel_change(moon_cell: &RefCell<MoonDimension>) {
    let mut moon = moon_cell.borrow_mut();
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

    let part_1_data = dimensions.clone();
    let part_2_data = dimensions;

    // Part 1
    for _ in 0..1_000 {
        for dimension in &part_1_data {
            perform_step(dimension);
        }
    }
    let total_energy: i32 = transpose_data(&part_1_data).iter().map(Moon::total_energy).sum();

    // Part 2
    let mut steps = Vec::new();
    for dimension in &part_2_data {
        steps.push(simulate(dimension));
    }
    let iterations_to_reset = lowest_common_multiple(steps);

    println!("Part 1: {}\nPart 2: {}", total_energy, iterations_to_reset);
}

fn transpose_data(data: &[Vec<RefCell<MoonDimension>>]) -> Vec<Moon> {
    izip!(data[0].clone(), data[1].clone(), data[2].clone()).map(|(x, y, z)| Moon { x: x.into_inner(), y: y.into_inner(), z: z.into_inner() }).collect()
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

#[derive(Clone, Copy, PartialEq, Eq)]
struct MoonDimension {
    pos: i32,
    vel: i32,
}

struct Moon {
    x: MoonDimension,
    y: MoonDimension,
    z: MoonDimension,
}

impl Moon {
    fn potential_energy(&self) -> i32 {
        self.x.pos.abs() + self.y.pos.abs() + self.z.pos.abs()
    }

    fn kinetic_energy(&self) -> i32 {
        self.x.vel.abs() + self.y.vel.abs() + self.z.vel.abs()
    }

    fn total_energy(&self) -> i32 {
        self.potential_energy() * self.kinetic_energy()
    }
}