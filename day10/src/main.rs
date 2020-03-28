#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]

use std::io;
use std::io::Read;
use std::fs::File;
use std::process;

fn main() {
    let mut map = load_map().unwrap_or_else(|err| {
        println!("Could not load input file!\n{:?}", err);
        process::exit(1);
    });
    
    let best_position = find_best_position(&mut map);
    let (pos_x, pos_y) = position_to_coords(&map, best_position);

    // Calculate the angle of every position on the map.
    let mut angles = Vec::new();
    for position in 0..(map.width * map.height) {
        let (x, y) = position_to_coords(&map, position);
        let (delta_x, delta_y) = (x - pos_x, y - pos_y);
        angles.push(angle(delta_x, delta_y));
    }

    map.positions[best_position].asteroid = false;

    let mut last_angle = 2.0 * std::f64::consts::PI;
    let mut asteroids_destroyed = 0;
    loop {
        let (asteroid, angle) = find_next_asteroid(&map, &angles, last_angle, false);
        destroy_asteroid(&mut map, best_position, asteroid);
        last_angle = angle;
        asteroids_destroyed += 1;
        let (final_x, final_y) = position_to_coords(&map, asteroid);
        let answer = (final_x * 100) + final_y;
        if asteroids_destroyed > 200 {
            println!("Part 2 answer: {}", answer);
            break;
        }
    }
}

// fn print_map(map: &Map, position: usize, asteroid: usize) {
//     let mut drawing = String::new();
//     for i in 0..(map.width * map.height) {
//         if i % map.width == 0 {
//             drawing.push('\n');
//         }
//         if i == position {
//             drawing.push('B');
//         } else if i == asteroid {
//             drawing.push('#');
//         } else if map.positions[i].asteroid {
//             drawing.push('o');
//         } else {
//             drawing.push(' ');
//         }
//     }
//     println!("{}", drawing);
// }

fn find_next_asteroid(map: &Map, angles: &[f64], last_angle: f64, allow_same_angle: bool) -> (usize, f64) {
    let mut next_asteroid = map.height * map.width;
    let mut best_angle = 2.0 * std::f64::consts::PI;
    let mut found_one = false;
    for asteroid in &map.asteroids {
        if !map.positions[*asteroid].asteroid { continue; } // Skip ones we've already blown up
        if !map.positions[*asteroid].visible { continue; } // Skip ones we can't see
        if angles[*asteroid] < last_angle { continue; } // It's counter-clockwise
        if !allow_same_angle && ((angles[*asteroid] - last_angle) < std::f64::EPSILON) { continue; } // It's behind the last one
        if angles[*asteroid] > best_angle { continue; } // We've already found a better one
        next_asteroid = *asteroid;
        best_angle = angles[*asteroid];
        found_one = true;
    }

    if !found_one { // We didn't find one - reset angle to 0 and try again.
        let (next, best) = find_next_asteroid(map, angles, 0.0, true);
        next_asteroid = next;
        best_angle = best;
    }

    (next_asteroid, best_angle)
}

// Blow up an asteroid and reveal one behind it, if any.
fn destroy_asteroid(map: &mut Map, position: usize, asteroid: usize) {
    assert!(map.positions[asteroid].visible);
    map.positions[asteroid].asteroid = false;

    let (pos_x, pos_y) = position_to_coords(&map, position);
    let (ast_x, ast_y) = position_to_coords(&map, asteroid);
    let (mut delta_x, mut delta_y) = (ast_x - pos_x, ast_y - pos_y);
    let gcd = greatest_common_denominator(delta_x.abs() as u32, delta_y.abs() as u32) as isize;
    delta_x /= gcd;
    delta_y /= gcd;
    let (mut x, mut y) = (ast_x, ast_y);
    loop {
        x += delta_x;
        y += delta_y;
        if (x >= map.width as isize) || (x < 0) || (y >= map.height as isize) || (y < 0) { break; }
        let possible_position = coords_to_position(&map, x, y);
        if map.positions[possible_position].asteroid {
            assert!(!map.positions[possible_position].visible);
            map.positions[possible_position].visible = true;
            break;
        }
    }
}

// Calculate the angle towards a co-ordinate, in radians from 0 to 2*PI, 
// with straight up being 0 and increasing clockwise.
fn angle(delta_x: isize, delta_y: isize) -> f64 {
    if (delta_x == 0) && (delta_y < 0) { // Straight up
        0.0
    } else {
        (-delta_x as f64).atan2(delta_y as f64) + std::f64::consts::PI
    }
}

fn find_best_position(map: &mut Map) -> usize {
    let mut best_position = 0;
    let mut most_visible_asteroids = 0;
    let mut best_positions = None;
    for possible_location in &map.asteroids {
        let (pos_x, pos_y) = position_to_coords(&map, *possible_location);
        let mut positions = map.positions.clone();

        for asteroid in &map.asteroids {
            let (ast_x, ast_y) = position_to_coords(&map, *asteroid);
            if (ast_x, ast_y) == (pos_x, pos_y) { continue; } // Skip our own asteroid
            let mut delta_x = ast_x - pos_x;
            let mut delta_y = ast_y - pos_y;
            let gcd = greatest_common_denominator(delta_x.abs() as u32, delta_y.abs() as u32) as isize;
            delta_x /= gcd;
            delta_y /= gcd;
            let (mut x, mut y) = (ast_x, ast_y);
            loop {
                x += delta_x;
                y += delta_y;
                if (x >= map.width as isize) || (x < 0) || (y >= map.height as isize) || (y < 0) { break; }
                positions[coords_to_position(&map, x, y)].visible = false;
            }
        }

        let mut visible_asteroids = 0;
        for asteroid in &map.asteroids {
            if asteroid == possible_location { continue; }
            if positions[*asteroid].visible { visible_asteroids += 1; }
        }
        if visible_asteroids > most_visible_asteroids {
            most_visible_asteroids = visible_asteroids;
            best_position = *possible_location;
            best_positions = Some(positions);
        }
    }

    //map.positions.clear();
    map.positions.copy_from_slice(&best_positions.unwrap());

    println!("Part 1: {}", most_visible_asteroids);
    best_position
}

fn greatest_common_denominator(u: u32, v: u32) -> u32 {
    if u == v { u }
    else if u == 0 { v }
    else if v == 0 { u }
    else if (!u & 1) != 0 { // u is even
        if (v & 1) == 0 { // both even
            greatest_common_denominator(u >> 1, v >> 1) << 1
        } else { // v is odd
            greatest_common_denominator(u >> 1, v)
        }
    } else if (!v & 1) != 0 { // u is odd, v is even
        greatest_common_denominator(u, v >> 1)
    } else if u > v { // Reduce larger argument
        greatest_common_denominator((u - v) >> 1, v)
    } else {
        greatest_common_denominator((v - u) >> 1, u)
    }
}

fn position_to_coords(map: &Map, position: usize) -> (isize, isize) {
    ((position % map.width) as isize, (position / map.width) as isize)
}

fn coords_to_position(map: &Map, x: isize, y: isize) -> usize {
    ((y * map.width as isize) + x) as usize
}

struct Map {
    width: usize,
    height: usize,
    positions: Vec<Position>,
    asteroids: Vec<usize>,
}

#[derive(Clone, Copy)]
struct Position {
    asteroid: bool,
    visible: bool,
}

impl Position {
    fn new(asteroid: bool) -> Self {
        Position {
            asteroid,
            visible: true,
        }
    }
}

fn load_map() -> Result<Map, io::Error> {
    let mut input = File::open("day10/input.txt")?;
    let mut data = String::new();
    input.read_to_string(&mut data)?;

    let mut positions = Vec::new();
    let mut asteroids = Vec::new();
    let mut position = 0;
    let mut height = 0;
    let mut width = 0;
    for line in data.lines() {
        height += 1;
        width = 0;
        for c in line.chars() {
            width += 1;
            let asteroid = match c {
                '#' => {
                    asteroids.push(position);
                    true
                },
                _ => false,
            };
            positions.push(Position::new(asteroid));
            position += 1;
        }
    }

    Ok(Map { width, height, positions, asteroids})
}