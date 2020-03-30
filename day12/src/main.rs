use std::cell::RefCell;

extern crate num;
extern crate primes;
#[macro_use]
extern crate itertools;

// Perform repeated iterations on a single dimension until all the moons are back to their
// original state in that dimension.
fn simulate(moons: &[RefCell<MoonDimension>]) -> u64 {
    let moons = moons.to_owned();
    let initial_state = moons.clone();
    let mut steps = 0;
    loop {
        perform_step(&moons);
        steps += 1;
        if moons == initial_state {
            break steps;
        }
    }
}

// Make a single step for all moons but in one dimension.
fn perform_step(moons: &[RefCell<MoonDimension>]) {
    for this_moon_cell in moons {
        for other_moon_cell in moons {
            if this_moon_cell as *const _ == other_moon_cell as *const _ {
                continue;
            }
            calc_vel_change(this_moon_cell, other_moon_cell);
        }
    }
    for this_moon_cell in moons {
        apply_vel_change(this_moon_cell);
    }
}

fn calc_vel_change(
    this_moon_cell: &RefCell<MoonDimension>,
    other_moon_cell: &RefCell<MoonDimension>,
) {
    let mut this_moon = this_moon_cell.borrow_mut();
    let other_moon = other_moon_cell.borrow();
    if other_moon.pos > this_moon.pos {
        this_moon.vel += 1;
    }
    if other_moon.pos < this_moon.pos {
        this_moon.vel -= 1;
    }
}

fn apply_vel_change(moon_cell: &RefCell<MoonDimension>) {
    let mut moon = moon_cell.borrow_mut();
    moon.pos += moon.vel;
}

fn main() {
    let start_time = std::time::Instant::now();

    // There's no relationship between the three dimensions - what happens in one is entirely
    // independent of what happens in the other two.  We're going to make use of that to
    // simulate them independently, so store them independently.
    let dimensions = vec![
        vec![
            RefCell::new(MoonDimension { pos: -10, vel: 0 }),
            RefCell::new(MoonDimension { pos: 5, vel: 0 }),
            RefCell::new(MoonDimension { pos: 3, vel: 0 }),
            RefCell::new(MoonDimension { pos: 1, vel: 0 }),
        ],
        vec![
            RefCell::new(MoonDimension { pos: -10, vel: 0 }),
            RefCell::new(MoonDimension { pos: 5, vel: 0 }),
            RefCell::new(MoonDimension { pos: 8, vel: 0 }),
            RefCell::new(MoonDimension { pos: 3, vel: 0 }),
        ],
        vec![
            RefCell::new(MoonDimension { pos: -13, vel: 0 }),
            RefCell::new(MoonDimension { pos: -9, vel: 0 }),
            RefCell::new(MoonDimension { pos: -16, vel: 0 }),
            RefCell::new(MoonDimension { pos: -3, vel: 0 }),
        ],
    ];

    // We're going to mutate the data, so use a different copy for each part.
    let part_1_data = dimensions.clone();
    let part_2_data = dimensions;

    // Part 1 - just perform a thousand iterations on the whole thing.
    for _ in 0..1_000 {
        for dimension in &part_1_data {
            perform_step(dimension);
        }
    }
    let total_energy: i32 = transpose_data(&part_1_data)
        .iter()
        .map(Moon::total_energy)
        .sum();

    // Part 2 - figure out how many steps required in each dimension to return to the initial
    // state (the dimensions are independent, rememeber).  The total steps for a complete
    // return to the initial state is then the lowest common multiple of the steps for each
    // individual dimension.  Because the dimensions are independent, let's run the
    // simulations on separate threads for a bit more juicy speed.
    let threads = part_2_data.iter().map(move |dimension| {
        let moved_dimension = dimension.to_vec();
        std::thread::spawn(move || simulate(&moved_dimension))
    }).collect::<Vec<_>>();

    let steps = threads.into_iter().map(|thread| thread.join().unwrap()).collect::<Vec<_>>();
    let iterations_to_reset = lowest_common_multiple(&steps);

    println!(
        "Part 1: {}\nPart 2: {}\nTime: {}ms",
        total_energy,
        iterations_to_reset,
        start_time.elapsed().as_millis()
    );
}

// It was better for part 2 to have dimensions containing moons. But part 1 wants moons containing
// dimensions, so switch them round.
fn transpose_data(data: &[Vec<RefCell<MoonDimension>>]) -> Vec<Moon> {
    izip!(data[0].clone(), data[1].clone(), data[2].clone())
        .map(|(x, y, z)| Moon {
            x: x.into_inner(),
            y: y.into_inner(),
            z: z.into_inner(),
        })
        .collect()
}

// Yuck, maths.  Stole this.
fn lowest_common_multiple(nums: &[u64]) -> u64 {
    let mut lcm = 1;

    let mut prime_factors = Vec::new();
    let mut unique_prime_factors = Vec::new();
    for num in nums {
        let factors = primes::factors(*num);
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
                if *factor == unique_factor {
                    count += 1;
                }
            }
            if count > max_count {
                max_count = count;
            }
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
