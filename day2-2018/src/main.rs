use std::io::Read;
use std::fs::File;

struct Result {
    doubles: usize,
    triples: usize,
}

fn main() {
    let mut input = File::open("day2-2018/input.txt").unwrap();
    let mut ids = String::new();
    input.read_to_string(&mut ids).unwrap();

    part_one(&ids);

    for (index, id1) in ids.lines().enumerate() {
        for id2 in ids.lines().skip(index + 1) {
            if id1.chars().count() != id2.chars().count() { continue; }

            for i in 0..id1.chars().count() {
                let str1 = id1.chars().take(i).collect::<String>() + &id1.chars().skip(i + 1).collect::<String>();
                let str2 = id2.chars().take(i).collect::<String>() + &id2.chars().skip(i + 1).collect::<String>();
                //println!("{} and {} - compare {} to {}", id1, id2, str1, str2);
                if str1 == str2 { println!("{}", str1); }
            }
        }
    }
}

fn part_one(ids: &String) {
    let result = ids.lines().fold(Result {doubles: 0, triples: 0}, |result_so_far, id_str| {
        let mut map = std::collections::HashMap::new();
        for c in id_str.chars() {
            let num = map.get(&c);
            if num.is_none() {}
            if let Some(num) = map.get(&c) {
                let new_num = num + 1;
                map.insert(c, new_num);
            } else {
                map.insert(c, 1);
            }
        }

        let mut double = false;
        let mut triple = false;
        for num in map.values() {
            if *num == 2 { double = true; }
            if *num == 3 { triple = true; }
        }

        println!("{}: {}, {}", id_str, double, triple);
        Result {
            doubles: result_so_far.doubles + (if double {1} else {0}),
            triples: result_so_far.triples + (if triple {1} else {0}),
        }
    });
    println!("{}", result.doubles * result.triples);
}