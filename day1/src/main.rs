use std::io;
use std::io::Read;
use std::fs::File;
use std::process;

fn main() {
    let modules = load_modules().unwrap_or_else(|err| {
        println!("Could not load input file!\n{:?}", err);
        process::exit(1);
    });
    let total_fuel: i32 = modules.iter().copied().map(fuel_for_module).sum();
    println!("Total fuel required: {}", total_fuel);
}

fn load_modules() -> Result<Vec<i32>, io::Error> {
    let mut input = File::open("day1/input.txt")?;
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