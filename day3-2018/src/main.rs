use std::io::{Read, Write};
use std::fs::File;
use std::collections::HashMap;

#[macro_use] extern crate itertools;
extern crate regex;

struct Claim {
    _id: usize,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
}

fn main() {
    let mut input = File::open("day3-2018/input.txt").unwrap();
    let mut claims = String::new();
    input.read_to_string(&mut claims).unwrap();

    let mut map = HashMap::new();
    let mut count = 0;
    let re = regex::Regex::new(r"#(\d+) @ (\d+),(\d+): (\d+)x(\d+)").unwrap();
    for cap in re.captures_iter(&claims) {
        let claim = Claim {
            _id: cap[1].parse().unwrap(),
            x: cap[2].parse().unwrap(),
            y: cap[3].parse().unwrap(),
            width: cap[4].parse().unwrap(),
            height: cap[5].parse().unwrap(),            
        };

        println!("Claim {}: ({}, {}) to ({}, {})", claim._id, claim.x, claim.y, claim.x + claim.width - 1, claim.y + claim.height - 1);

        for (x, y) in iproduct!((claim.x..claim.x + claim.width), (claim.y..claim.y + claim.height)) {
            let old_count = map.get(&(x, y));
            if old_count.is_none() {
                map.insert((x, y), 1);
            } else {
                let new_count = old_count.unwrap() + 1;
                if new_count == 2 { count += 1; }
                map.insert((x, y), new_count);
            }
        }
    }

    let mut output = File::create("day3-2018/output.txt").unwrap();
    for (x, y) in iproduct!((0..2000), (0..2000)) {
        if y == 0 { output.write_all(b"\n").unwrap(); }
        if let Some(num) = map.get(&(x, y)) {
            if *num == 1 {
                output.write_all(b"#").unwrap(); 
            } else { 
                output.write_all(b"X").unwrap();
            }
        } else {
            output.write_all(b" ").unwrap();
        }
    }

    println!("{}", count);
}
