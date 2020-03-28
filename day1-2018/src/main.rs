use std::io::Read;
use std::fs::File;
use std::process;

fn main() {
    let mut input = File::open("day1-2018/input.txt").unwrap();
    let mut frequencies = String::new();
    input.read_to_string(&mut frequencies).unwrap();
    let mut seen_freqs = std::collections::HashSet::new();
    frequencies
        .lines()
        .cycle()
        .fold(0, |freq, x| {
            let new_freq = freq + x.parse::<i32>().unwrap();
            if !seen_freqs.insert(new_freq) {
                println!("{}", new_freq);
                process::exit(1);
            }
            new_freq
        });
}
