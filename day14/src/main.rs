#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};
use std::io::{self, Read};

extern crate math;

const ONE_TRILLION: u64 = 1_000_000_000_000;

fn main() {
    let start_time = std::time::Instant::now();

    // `load_reactions` gives us a map of chemical name to a representation of that chemical.
    // But it goes beyond that - behind the scenes, the chemicals are all connected, and
    // understand their dependencies on one another.  Each chemical understands what it needs
    // to make a certain quantity of itself, and what quantity of itself is needed to make
    // what other chemicals need.
    let map = load_reactions("day14/input.txt").unwrap_or_else(|err| {
        println!("Could not load input file!\n{:?}", err);
        std::process::exit(1);
    });

    // To start with, there's no actual demand for anything, because while every chemical
    // understands what WOULD be needed to make some of itself, its parent chemical(s)
    // haven't asked it for anything.  To get things going, we're going to construct a
    // phantom parent of FUEL that asks for one FUEL.
    let fuel_cell = map.get("FUEL").unwrap();
    let mut fuel = fuel_cell.borrow_mut();
    let req = Rc::new(RefCell::new(Requirement {
        // This says: I make one of myself with one FUEL, and one of me is needed.
        chemical: Rc::downgrade(&fuel_cell),
        reaction_quantity: 1,
        needed: 1,
    }));
    fuel.needed_by.push(Rc::downgrade(&req));

    // Trigger calculation.
    recalculate_requirements(&mut fuel);

    // That's flowed all the way down, so throughout the tree we can see how much of each
    // chemical is required - and everything ultimately comes from ORE.  This is part 1 -
    // how much ORE is needed to make one FUEL.
    let ore_cell = map.get("ORE").unwrap();
    let ore_for_one_fuel = ore_cell.borrow().needed;

    // Part 2 asks: how much FUEL can be made with a trillion ORE?  Let's start with a
    // naive ratio - i.e. if we needed a billion ORE for one FUEL, let's see how much ORE
    // is needed for a thousand FUEL.
    let mut fuel_amount = (ONE_TRILLION / ore_for_one_fuel) + 1;
    req.borrow_mut().needed = fuel_amount;
    recalculate_requirements(&mut fuel);
    let initial_ore = ore_cell.borrow().needed;

    // That won't be anywhere near correct - the graph of ORE to FUEL will be nothing like
    // a straight line due to the complexity of the dependencies.  We can get a whole lot
    // closer, though, by adjusting by the ratio by which we were off first time.  E.g. if
    // we wound up needing half a trillion ORE with the FUEL we guessed, let's try doubling
    // the FUEL.
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

    println!(
        "Part 1: {}\nPart 2: {}\nTime: {}us",
        ore_for_one_fuel,
        fuel_amount,
        start_time.elapsed().as_micros()
    );
}

// Figure out how many of the specified chemical are needed to satisfy its parents' requirements.
// Operates recursively to update its child chemicals.
fn recalculate_requirements(chemical: &mut Chemical) {
    // The demand for this chemical is the sum of its `needed_by` list.
    chemical.needed = chemical
        .needed_by
        .iter()
        .map(|weak| weak.upgrade().unwrap().borrow().needed)
        .sum();

    // We can't necessarily generate an arbitrary quantity of this chemical, however - each
    // reaction generates a specific amount.  See how many reactions generate what we need
    // with minimum wastage.
    let reactions =
        math::round::ceil(chemical.needed as f64 / chemical.creates_quantity as f64, 0) as u64;

    // Update our requirements on our children accordingly, and recurse to them.
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

impl Chemical {
    fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            reqs: Vec::new(),
            creates_quantity: 0,
            needed: 0,
            needed_by: Vec::new(),
        }))
    }
}

struct Requirement {
    chemical: Weak<RefCell<Chemical>>,
    reaction_quantity: u64,
    needed: u64,
}

impl Requirement {
    fn new(chemical: &Rc<RefCell<Chemical>>, quantity: u64) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            chemical: Rc::downgrade(chemical),
            reaction_quantity: quantity,
            needed: 0,
        }))
    }
}

fn load_reactions(source_file: &str) -> Result<HashMap<String, Rc<RefCell<Chemical>>>, io::Error> {
    let mut input = std::fs::File::open(source_file)?;
    let mut reactions = String::new();
    input.read_to_string(&mut reactions)?;

    let mut map: HashMap<String, Rc<RefCell<Chemical>>> = HashMap::new();

    for reaction in reactions.lines() {
        let mut sides = reaction.split(" => ");
        let inputs = sides
            .next()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Invalid input line: {}", reaction),
                )
            })?
            .split(", ");
        let output = sides.next().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Invalid input line: {}", reaction),
            )
        })?;

        let (output_quantity, output_name) = parse_quantity(output)?;
        let output_chemical_cell = get_chemical(&mut map, output_name);
        let mut output_chemical = output_chemical_cell.borrow_mut();
        output_chemical.creates_quantity = output_quantity;

        for input in inputs {
            let (input_quantity, input_name) = parse_quantity(input)?;
            let input_chemical_cell = get_chemical(&mut map, input_name);
            let req = Requirement::new(&input_chemical_cell, input_quantity);
            output_chemical.reqs.push(req.clone());
            let mut input_chemical = input_chemical_cell.borrow_mut();
            input_chemical.needed_by.push(Rc::downgrade(&req));
        }
    }

    Ok(map)
}

fn parse_quantity(input: &str) -> Result<(u64, &str), io::Error> {
    let mut inputs = input.split(' ');
    let quantity_str = inputs.next().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Invalid input data: {}", input),
        )
    })?;
    let quantity = quantity_str
        .parse::<u64>()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    let name = inputs.next().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Invalid input data: {}", input),
        )
    })?;
    Ok((quantity, name))
}

fn get_chemical(
    map: &mut HashMap<String, Rc<RefCell<Chemical>>>,
    name: &str,
) -> Rc<RefCell<Chemical>> {
    map.entry(name.to_string()).or_insert_with(Chemical::new).clone()
}
