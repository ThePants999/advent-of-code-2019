#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]

use std::io::{self, Read};

// Once again, today's code is verbose, but both performant and (hopefully) easy to follow.

fn main() {
    let start_time = std::time::Instant::now();

    let mut map = load_map().unwrap_or_else(|err| {
        println!("Could not load input file!\n{:?}", err);
        std::process::exit(1);
    });

    // Part 1 - find the best location for a base.  Also updates visibility as calculated from
    // that position.
    let best_position = find_best_position(&mut map);
    let (pos_x, pos_y) = position_to_coords(&map, best_position);

    // `find_best_position` already calculated the visibility of all other asteroids from the
    // chosen location - we also need to calculate the angles.
    calculate_angles(&mut map, pos_x, pos_y);

    // Let's avoid having to continually skip over ourselves!
    map.positions[best_position].asteroid = false;

    // IMMA FIRIN MAH LAZOR
    let final_asteroid = destroy_asteroids(&mut map, 200, pos_x, pos_y);
    let (final_x, final_y) = position_to_coords(&map, final_asteroid);
    let answer = (final_x * 100) + final_y;
    println!(
        "Part 2: {}\nTime: {}ms",
        answer,
        start_time.elapsed().as_millis()
    );
}

// Destroy `quantity` asteroids, and return the last asteroid destroyed.
fn destroy_asteroids(map: &mut Map, quantity: usize, source_x: isize, source_y: isize) -> usize {
    // Starting from a negative angle means the first asteroid we find will be one with angle 0.0.
    let mut last_angle = -1.0;
    let mut asteroids_destroyed = 0;

    loop {
        let (asteroid, angle) = find_next_asteroid(&map, last_angle);
        destroy_asteroid(map, source_x, source_y, asteroid);
        last_angle = angle;

        asteroids_destroyed += 1;
        if asteroids_destroyed == quantity {
            break asteroid;
        }
    }
}

fn _print_map(map: &Map, position: usize, asteroid: usize) {
    let mut drawing = String::new();
    for i in 0..(map.width * map.height) {
        if i % map.width == 0 {
            drawing.push('\n');
        }
        if i == position {
            drawing.push('B');
        } else if i == asteroid {
            drawing.push('#');
        } else if map.positions[i].asteroid {
            drawing.push('o');
        } else {
            drawing.push(' ');
        }
    }
    println!("{}", drawing);
}

// Given the angle of the last asteroid we destroyed, figure out the next one we find rotating
// clockwise. It's the visible asteroid whose angle is strictly larger than the last one, but by
// the least amount. When we complete a circle, we'll fail to find one, in which case we start
// again from a negative value (which makes asteroids with angle 0.0 valid).
fn find_next_asteroid(map: &Map, last_angle: f64) -> (usize, f64) {
    let (next_asteroid, best_angle) = map
        .asteroids
        .iter()
        .filter(|asteroid| {
            // Filter out asteroids that are...
            map.positions[**asteroid].asteroid && // already destroyed
            map.positions[**asteroid].visible && // hidden behind still-existing ones
            map.positions[**asteroid].angle > last_angle // counter-clockwise from here
        })
        .fold((None, 2.0 * std::f64::consts::PI), |acc, asteroid| {
            if map.positions[*asteroid].angle < acc.1 {
                // New best asteroid
                (Some(*asteroid), map.positions[*asteroid].angle)
            } else {
                acc
            }
        });

    if let Some(asteroid) = next_asteroid {
        (asteroid, best_angle)
    } else {
        // We didn't find one - reset the angle to negative and try again.
        find_next_asteroid(map, -1.0)
    }
}

// Blow up an asteroid and reveal one behind it, if any.
fn destroy_asteroid(map: &mut Map, source_x: isize, source_y: isize, asteroid: usize) {
    assert!(map.positions[asteroid].visible);
    map.positions[asteroid].asteroid = false;

    let (target_x, target_y) = position_to_coords(&map, asteroid);
    let (delta_x, delta_y) = calculate_visibility_delta(source_x, source_y, target_x, target_y);

    let (mut x, mut y) = (target_x, target_y);
    loop {
        x += delta_x;
        y += delta_y;
        if coords_outside_map(map, x, y) {
            break;
        }

        let possible_position = coords_to_position(&map, x, y);
        if map.positions[possible_position].asteroid {
            assert!(!map.positions[possible_position].visible);
            map.positions[possible_position].visible = true;
            break;
        }
    }
}

// Update all asteroids with a calculated angle from the specified source.
fn calculate_angles(map: &mut Map, source_x: isize, source_y: isize) {
    for position in 0..(map.width * map.height) {
        let (x, y) = position_to_coords(&map, position);
        let (delta_x, delta_y) = (x - source_x, y - source_y);
        map.positions[position].angle = angle(delta_x, delta_y);
    }
}

// Calculate the angle towards a co-ordinate, in radians from 0 to 2*PI,
// with straight up being 0 and increasing clockwise.
fn angle(delta_x: isize, delta_y: isize) -> f64 {
    if (delta_x == 0) && (delta_y < 0) {
        // Straight up
        0.0
    } else {
        (-delta_x as f64).atan2(delta_y as f64) + std::f64::consts::PI
    }
}

// The part 1 problem - find the asteroid that can see the most other asteroids. We take a pretty
// brute-force approach here:
// -  For each asteroid:
//    -  Start with a fresh map
//    -  For each other asteroid, cross off every location that's rendered invisible behind it
//    -  Count how many visible asteroids are left
//    -  Keep track of the best we've found.
//
// Determining which asteroids
fn find_best_position(map: &mut Map) -> usize {
    let mut best_position = 0;
    let mut most_visible_asteroids = 0;
    let mut best_positions = None;
    for possible_location in &map.asteroids {
        let (pos_x, pos_y) = position_to_coords(&map, *possible_location);
        let mut positions = map.positions.clone();

        for asteroid in &map.asteroids {
            mark_hidden_positions(map, &mut positions, pos_x, pos_y, *asteroid);
        }

        let visible_asteroids = map
            .asteroids
            .iter()
            .filter(|asteroid| positions[**asteroid].visible)
            .count()
            - 1; // Knock off one to avoid including ourselves
        if visible_asteroids > most_visible_asteroids {
            most_visible_asteroids = visible_asteroids;
            best_position = *possible_location;
            best_positions = Some(positions);
        }
    }

    map.positions.copy_from_slice(&best_positions.unwrap());

    println!("Part 1: {}", most_visible_asteroids);
    best_position
}

// Figure out and cross off positions that are hidden from view by a specified asteroid from a
// specified starting location.
//
// The mechanic for this is:
// -  Figure out the coordinate delta from source to target
// -  Calculate at what further deltas beyond that target positions are hidden
// -  From the target, take adjusted delta steps, marking each one as invisible, until we
//    reach the edge of the map.
fn mark_hidden_positions(
    map: &Map,
    positions: &mut Vec<Position>,
    source_x: isize,
    source_y: isize,
    asteroid: usize,
) {
    let (ast_x, ast_y) = position_to_coords(&map, asteroid);
    if (ast_x, ast_y) == (source_x, source_y) {
        return; // Skip our own asteroid
    }

    let (delta_x, delta_y) = calculate_visibility_delta(source_x, source_y, ast_x, ast_y);

    let (mut x, mut y) = (ast_x, ast_y);
    loop {
        x += delta_x;
        y += delta_y;
        if coords_outside_map(map, x, y) {
            break;
        }
        positions[coords_to_position(&map, x, y)].visible = false;
    }
}

fn coords_outside_map(map: &Map, x: isize, y: isize) -> bool {
    (x >= map.width as isize) || (x < 0) || (y >= map.height as isize) || (y < 0)
}

// Looking from a starting point towards an asteroid, figure out the steps beyond that asteroid in
// which positions are hidden by that asteroid. E.g. if we're looking at an asteroid that's [6, -3]
// away, it hides everything that's in steps of [2, -1] beyond it.
//
// That's calculated by dividing the deltas by their greatest common denominator.
fn calculate_visibility_delta(
    source_x: isize,
    source_y: isize,
    target_x: isize,
    target_y: isize,
) -> (isize, isize) {
    let delta_x = target_x - source_x;
    let delta_y = target_y - source_y;
    let gcd = greatest_common_denominator(delta_x.abs() as u32, delta_y.abs() as u32) as isize;
    let adjusted_delta_x = delta_x / gcd;
    let adjusted_delta_y = delta_y / gcd;
    (adjusted_delta_x, adjusted_delta_y)
}

// I stole this.  I don't do maths.
fn greatest_common_denominator(u: u32, v: u32) -> u32 {
    if u == v {
        u
    } else if u == 0 {
        v
    } else if v == 0 {
        u
    } else if (!u & 1) != 0 {
        // u is even
        if (v & 1) == 0 {
            // both even
            greatest_common_denominator(u >> 1, v >> 1) << 1
        } else {
            // v is odd
            greatest_common_denominator(u >> 1, v)
        }
    } else if (!v & 1) != 0 {
        // u is odd, v is even
        greatest_common_denominator(u, v >> 1)
    } else if u > v {
        // Reduce larger argument
        greatest_common_denominator((u - v) >> 1, v)
    } else {
        greatest_common_denominator((v - u) >> 1, u)
    }
}

fn position_to_coords(map: &Map, position: usize) -> (isize, isize) {
    (
        (position % map.width) as isize,
        (position / map.width) as isize,
    )
}

fn coords_to_position(map: &Map, x: isize, y: isize) -> usize {
    ((y * map.width as isize) + x) as usize
}

// A record of what we know about the asteroid field. The `Positions` vector represents this 2D
// space folded into 1D as a series of rows, hence the two functions above to convert between
// vector index and co-ordinates.
struct Map {
    width: usize,
    height: usize,
    positions: Vec<Position>,
    asteroids: Vec<usize>,
}

// A location in space.
// -  `asteroid` records whether there's an asteroid in it.  Initially this is loaded from the
//    input map, but mutated once we start destroying asteroids in part 2.
// -  `visible` and `angle` are N/A until we've determined the location for the base in part 1.
//    They're then calculated from the perspective of the selected location.  They're only
//    applicable to locations with asteroids.
#[derive(Clone, Copy)]
struct Position {
    asteroid: bool,
    visible: bool,
    angle: f64,
}

impl Position {
    fn new(asteroid: bool) -> Self {
        Position {
            asteroid,
            visible: true,
            angle: 0.0,
        }
    }
}

fn load_map() -> Result<Map, io::Error> {
    let mut input = std::fs::File::open("day10/input.txt")?;
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
                }
                _ => false,
            };
            positions.push(Position::new(asteroid));
            position += 1;
        }
    }

    Ok(Map {
        width,
        height,
        positions,
        asteroids,
    })
}
