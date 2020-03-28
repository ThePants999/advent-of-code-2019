use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::collections::HashMap;

use std::fs::File;
use std::io;
use std::io::Read;
use std::process;

extern crate math;

const ONE_TRILLION: u64 = 1000000000000;

fn main() {
    let map = load_reactions("day14/input.txt").unwrap_or_else(|err| {
        println!("Could not load input file!\n{:?}", err);
        process::exit(1);
    });

    let fuel_cell = map.get("FUEL").unwrap_or_else(|| {
        println!("Reactions list didn't contain fuel!");
        process::exit(1);
    });

    // Record a need for one fuel so that the calculation generates it.
    let mut fuel = fuel_cell.borrow_mut();
    let req = Rc::new(RefCell::new(Requirement {
        chemical: Rc::downgrade(&fuel_cell),
        reaction_quantity: 1,
        needed: 1,
    }));
    fuel.needed_by.push(Rc::downgrade(&req));

    // Trigger calculation.
    recalculate_requirements(&mut fuel);

    let ore_cell = map.get("ORE").unwrap_or_else(|| {
        println!("Reactions list didn't contain ore!");
        process::exit(1);
    });

    let ore_for_one_fuel = ore_cell.borrow().needed;
    println!("ORE required for one FUEL: {}", ore_for_one_fuel);

    // That's a starting point for how much fuel we can generate with a trillion ORE,
    // but efficiencies will mean we can generate more. Iterate. Firstly, let's work on
    // getting close.
    let mut fuel_amount = (ONE_TRILLION / ore_for_one_fuel) + 1;
    req.borrow_mut().needed = fuel_amount;
    recalculate_requirements(&mut fuel);
    let initial_ore = ore_cell.borrow().needed;
    let correction_factor = ONE_TRILLION as f64 / initial_ore as f64;
    fuel_amount = (fuel_amount as f64 * correction_factor) as u64;

    // Now we can iterate one by one until correct.
    loop {
        req.borrow_mut().needed = fuel_amount;
        recalculate_requirements(&mut fuel);
        let ore_required = ore_cell.borrow().needed;
        if ore_required > ONE_TRILLION {
            fuel_amount -= 1;
            break;
        } else {
            fuel_amount += 1;
        }
    }

    println!("Fuel with a trillion ore: {}", fuel_amount);
}

fn recalculate_requirements(chemical: &mut Chemical) {
    chemical.needed = chemical.needed_by.iter().map(|weak| { weak.upgrade().unwrap().borrow().needed }).sum();

    let reactions = math::round::ceil(chemical.needed as f64 / chemical.creates_quantity as f64, 0) as u64;
    for req_cell in &chemical.reqs {
        let mut req = req_cell.borrow_mut();
        req.needed = reactions * req.reaction_quantity;
        drop(req);
        let req = req_cell.borrow();
        let req_chemical_cell = req.chemical.upgrade().unwrap();
        let mut req_chemical = req_chemical_cell.borrow_mut();
        recalculate_requirements(&mut req_chemical);
    }    
}

struct Chemical {
    reqs: Vec<Rc<RefCell<Requirement>>>,
    creates_quantity: u64,
    needed: u64,
    needed_by: Vec<Weak<RefCell<Requirement>>>,
}

struct Requirement {
    chemical: Weak<RefCell<Chemical>>,
    reaction_quantity: u64,
    needed: u64,
}

fn load_reactions(source_file: &str) -> Result<HashMap<String, Rc<RefCell<Chemical>>>, io::Error> {
    let mut input = File::open(source_file)?;
    let mut reactions = String::new();
    input.read_to_string(&mut reactions)?;

    let mut map: HashMap<String, Rc<RefCell<Chemical>>> = HashMap::new();

    for reaction in reactions.lines() {
        let mut sides = reaction.split(" => ");
        let inputs = sides.next().ok_or(io::Error::new(io::ErrorKind::InvalidInput, format!("Invalid input line: {}", reaction)))?.split(", ");
        let output = sides.next().ok_or(io::Error::new(io::ErrorKind::InvalidInput, format!("Invalid input line: {}", reaction)))?;

        let (output_quantity, output_name) = parse_quantity(output)?;
        let output_chemical_cell = get_chemical(&mut map, output_name);
        let mut output_chemical = output_chemical_cell.borrow_mut();
        output_chemical.creates_quantity = output_quantity;

        for input in inputs {
            let (input_quantity, input_name) = parse_quantity(input)?;
            let input_chemical_cell = get_chemical(&mut map, input_name);
            let req = Rc::new(RefCell::new(Requirement {
                chemical: Rc::downgrade(&input_chemical_cell),
                reaction_quantity: input_quantity,
                needed: 0,
            }));
            output_chemical.reqs.push(req.clone());
            let mut input_chemical = input_chemical_cell.borrow_mut();
            input_chemical.needed_by.push(Rc::downgrade(&req));
        }
    }

    Ok(map)
}

fn parse_quantity(input: &str) -> Result<(u64, &str), io::Error> {
    let mut inputs = input.split(' ');
    let quantity_str = inputs.next().ok_or(io::Error::new(io::ErrorKind::InvalidInput, format!("Invalid input data: {}", input)))?;
    let quantity = quantity_str.parse::<u64>().map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    let name = inputs.next().ok_or(io::Error::new(io::ErrorKind::InvalidInput, format!("Invalid input data: {}", input)))?;
    Ok((quantity, name))
}

fn get_chemical(map: &mut HashMap<String, Rc<RefCell<Chemical>>>, name: &str) -> Rc<RefCell<Chemical>> {
    if !map.contains_key(name) {
        let chemical = Chemical {
            reqs: Vec::new(),
            creates_quantity: 0,
            needed: 0,
            needed_by: Vec::new(),
        };
        map.insert(name.to_string(), Rc::new(RefCell::new(chemical)));
    }
    map.get(name).unwrap().clone()
}