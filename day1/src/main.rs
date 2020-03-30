use std::io::{self, Read};

fn main() {
    let start_time = std::time::Instant::now();

    let modules = load_modules().unwrap_or_else(|err| {
        println!("Could not load input file!\n{:?}", err);
        std::process::exit(1);
    });
    
    let part_1_fuel: i32 = modules.iter().copied().map(|module| fuel_for_weight(module, false)).sum();
    let part_2_fuel: i32 = modules.iter().copied().map(|module| fuel_for_weight(module, true)).sum();
    println!("Part 1: {}\nPart 2: {}\nTime: {}us", part_1_fuel, part_2_fuel, start_time.elapsed().as_micros());
}

fn load_modules() -> Result<Vec<i32>, io::Error> {
    let mut input = std::fs::File::open("day1/input.txt")?;
    let mut modules = String::new();
    input.read_to_string(&mut modules)?;
    modules
        .lines()
        .map(|line| line.parse::<i32>()
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err)))
        .collect()
}

fn fuel_for_weight(weight: i32, include_fuel_for_fuel: bool) -> i32 {
    let mut total_fuel = 0;
    let mut weight_just_added = weight;
    loop {
        match (weight_just_added / 3) - 2 {
            fuel if fuel >= 0 => {
                weight_just_added = fuel;
                total_fuel += fuel;
            },
            _ => break,
        }
        if !include_fuel_for_fuel { break; }
    }
    total_fuel
}