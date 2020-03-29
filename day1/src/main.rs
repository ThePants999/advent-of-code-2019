use std::io::{self, Read};

fn main() {
    let start_time = std::time::Instant::now();

    let modules = load_modules().unwrap_or_else(|err| {
        println!("Could not load input file!\n{:?}", err);
        std::process::exit(1);
    });
    
    let part_1_fuel: i32 = modules.iter().copied().map(fuel_for_weight).sum();
    let part_2_fuel: i32 = modules.iter().copied().map(fuel_for_module).sum();
    println!("Part 1: {}\nPart 2: {}\nTime: {}ms", part_1_fuel, part_2_fuel, start_time.elapsed().as_millis());
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

fn fuel_for_module(module_mass: i32) -> i32 {
    let base_fuel = fuel_for_weight(module_mass);
    base_fuel + fuel_for_fuel(base_fuel)
}

fn fuel_for_fuel(weight: i32) -> i32 {
    match fuel_for_weight(weight) {
        0 => 0,
        fuel => fuel + fuel_for_fuel(fuel)
    }
}

fn fuel_for_weight(weight: i32) -> i32 {
    match (weight / 3) - 2 {
        fuel if fuel >= 0 => fuel,
        _ => 0
    }
}