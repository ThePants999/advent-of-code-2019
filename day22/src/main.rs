use std::io;
use std::io::Read;
use std::fs::File;
use std::process;

use std::collections::VecDeque;

const DECK_SIZE: i128 = 119315717514047;
const NUM_SHUFFLES: i128 = 101741582076661;

// All hail https://codeforces.com/blog/entry/72593, without which I would never
// have been able to do this.

fn main() {
    //let mut deck: Vec<u32> = (0..DECK_SIZE as u32).collect();
    let instructions = load_instructions().unwrap_or_else(|err| {
        println!("Could not load input file!\n{:?}", err);
        process::exit(1);
    });

    let mut f = (1, 0);
    for technique in &instructions {
        f = compose(f, translate(technique));
        //deck = apply_technique(deck, technique);
    }

    let big_f = pow_compose(f, NUM_SHUFFLES);
    let x = 2020;
    let card = mod_divide(x - big_f.1, big_f.0, DECK_SIZE);
    println!("{}", card);
}

fn _make_positive(num: i128) -> i128 {
    if num >= 0 { num } else { num + DECK_SIZE }
}

fn translate(technique: &Techniques) -> (i128, i128) {
    match technique {
        Techniques::DealIntoNewStack => (-1, -1),
        Techniques::Cut(n) => (1, -(*n)),
        Techniques::DealWithIncrement(n) => (*n, 0),
    }
}

fn compose((a, b): (i128, i128), (c, d): (i128, i128)) -> (i128, i128) {
    ((a * c) % DECK_SIZE, ((b * c) + d) % DECK_SIZE)
}

fn pow_compose(mut f: (i128, i128), mut k: i128) -> (i128, i128) {
    let mut g = (1, 0);
    while k > 0 {
        if k % 2 == 1 { g = compose(g, f); }
        k /= 2;
        f = compose(f, f);
    }
    g
}

fn pow_mod(mut x: i128, mut n: i128, m: i128) -> i128 {
    let mut y = 1;
    while n > 0 {
        if n % 2 == 1 { y = (y * x) % m; }
        n /= 2;
        x = (x * x) % m;
    }
    y
}

fn mod_divide(numerator: i128, denominator: i128, m: i128) -> i128 {
    (numerator * pow_mod(denominator, m-2, m)) % m
}

enum Techniques {
    DealIntoNewStack,
    Cut(i128),
    DealWithIncrement(i128),
}

fn _apply_technique(mut deck: Vec<u32>, technique: &Techniques) -> Vec<u32> {
    match technique {
        Techniques::DealIntoNewStack => {
            deck.reverse();
            deck
        },
        Techniques::Cut(count) if *count >= 0 => {
            cut_deck(&deck, *count as usize)
        },
        Techniques::Cut(count) => {
            cut_deck(&deck, deck.len() - count.abs() as usize)
        }
        Techniques::DealWithIncrement(increment) => {
            let mut old_deck = VecDeque::from(deck.clone());
            let mut position: i128 = 0;
            let deck_len = deck.len() as i128;
            for _ in 0..deck_len {
                deck[position as usize] = old_deck.pop_front().unwrap();
                position += increment;
                if position >= deck_len { position -= deck_len; }
                else if position < 0 { position += deck_len; }
            }
            deck
        }
    }
}

#[allow(dead_code)]
fn cut_deck(deck: &Vec<u32>, count: usize) -> Vec<u32> {
    let mut new_deck = deck[count..].to_vec();
    new_deck.extend_from_slice(&deck[..count]);
    new_deck
}

const DEAL_INTO_NEW_STACK: &str = "deal into new stack";
const CUT: &str = "cut ";
const DEAL_WITH_INCREMENT: &str = "deal with increment ";

fn load_instructions() -> Result<Vec<Techniques>, io::Error> {
    let mut input = File::open("day22/input.txt")?;
    let mut instructions = String::new();
    input.read_to_string(&mut instructions)?;

    let mut techniques = Vec::new();
    for line in instructions.lines() {
        if line.starts_with(DEAL_INTO_NEW_STACK) {
            techniques.push(Techniques::DealIntoNewStack);
        } else if line.starts_with(CUT) {
            let count = line[CUT.len()..].parse::<i128>().map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
            techniques.push(Techniques::Cut(count));
        } else if line.starts_with(DEAL_WITH_INCREMENT) {
            let increment = line[DEAL_WITH_INCREMENT.len()..].parse::<i128>().map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
            techniques.push(Techniques::DealWithIncrement(increment));
        }
    }

    Ok(techniques)
}

fn _find_card(deck: &Vec<u32>, card: u32) -> Option<usize> {
    for (index, deck_card) in deck.iter().enumerate() {
        if *deck_card == card { return Some(index); }
    }
    None
}