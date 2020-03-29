use std::cmp;
use std::io::{self, Read};

fn main() {
    let start_time = std::time::Instant::now();

    let wires = load_wires().unwrap_or_else(|err| {
        println!("Could not load input file!\n{:?}", err);
        std::process::exit(1);
    });

    // Could easily extend to arbitrary number of wires, but let's KISS.
    let mut manhattan_distances = Vec::new();
    let mut wire_lengths = Vec::new();
    for first in &wires[1].segments {
        for second in &wires[0].segments {
            if let Some(int) = calculate_intersection(&first, &second) {
                if (int.coords.x == 0) && (int.coords.y == 0) {
                    continue;
                }
                manhattan_distances.push(int.manhattan_distance);
                wire_lengths.push(int.wire_length);
            }
        }
    }

    println!(
        "Shortest Manhattan: {}\nShortest length: {}\nTime: {}ms",
        manhattan_distances.iter().min().unwrap(),
        wire_lengths.iter().min().unwrap(),
        start_time.elapsed().as_millis()
    );
}

enum Directions {
    Up,
    Down,
    Left,
    Right,
}

#[derive(PartialEq)]
enum Orientations {
    Horizontal,
    Vertical,
}

#[derive(PartialEq, Copy, Clone)]
struct Coordinates {
    x: i32,
    y: i32,
}

struct Segment {
    length_before: i32,
    orientation: Orientations,
    start: Coordinates,
    end: Coordinates,
}

#[allow(dead_code)]
struct Wire {
    id: i32,
    segments: Vec<Segment>,
}

struct Intersection {
    coords: Coordinates,
    manhattan_distance: i32,
    wire_length: i32,
}

fn calculate_intersection(first: &Segment, second: &Segment) -> Option<Intersection> {
    find_intersection(first, second).map(|coords| {
        let first_length = manhattan_distance(coords, first.start) + first.length_before;
        let second_length = manhattan_distance(coords, second.start) + second.length_before;
        Intersection {
            coords,
            manhattan_distance: manhattan_distance(coords, Coordinates { x: 0, y: 0 }),
            wire_length: first_length + second_length,
        }
    })
}

fn find_intersection(first: &Segment, second: &Segment) -> Option<Coordinates> {
    // I'm going to be cheeky and ignore the possibility of two horizontal or two
    // vertical segments intersecting along part of their length.
    //
    // Also, yuuuuck.
    if first.orientation == second.orientation {
        if (first.start == second.start) || (first.start == second.end) {
            return Some(first.start);
        } else if (first.end == second.start) || (first.end == second.end) {
            return Some(first.end);
        } else {
            return None;
        }
    } else if first.orientation == Orientations::Horizontal {
        if (cmp::min(first.start.x, first.end.x) <= second.start.x)
            && (cmp::max(first.start.x, first.end.x) >= second.start.x)
            && (cmp::min(second.start.y, second.end.y) <= first.start.y)
            && (cmp::max(second.start.y, second.end.y) >= first.start.y)
        {
            return Some(Coordinates {
                x: second.start.x,
                y: first.start.y,
            });
        }
    } else if (cmp::min(first.start.y, first.end.y) <= second.start.y)
        && (cmp::max(first.start.y, first.end.y) >= second.start.y)
        && (cmp::min(second.start.x, second.end.x) <= first.start.x)
        && (cmp::max(second.start.x, second.end.x) >= first.start.x)
    {
        return Some(Coordinates {
            x: first.start.x,
            y: second.start.y,
        });
    }

    None
}

fn manhattan_distance(first: Coordinates, second: Coordinates) -> i32 {
    (second.x - first.x).abs() + (second.y - first.y).abs()
}

fn load_wires() -> Result<Vec<Wire>, io::Error> {
    let mut input_file = std::fs::File::open("day3/input.txt")?;
    let mut input = String::new();
    input_file.read_to_string(&mut input)?;

    let mut id = 0;
    let mut wires: Vec<Wire> = Vec::new();
    for line in input.lines() {
        let mut length = 0;
        let mut x = 0;
        let mut y = 0;
        let segments = line
            .split(',')
            .map(|cmd_text| {
                let mut chars = cmd_text.chars();
                let direction = match chars.next() {
                    Some('U') => Directions::Up,
                    Some('D') => Directions::Down,
                    Some('L') => Directions::Left,
                    Some('R') => Directions::Right,
                    Some(invalid_char) => {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            format!("Invalid direction: {}", invalid_char),
                        ))
                    }
                    None => {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            String::from("Empty command"),
                        ))
                    }
                };
                let distance = chars.as_str().parse::<i32>().map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("Invalid distance: {} ({})", chars.as_str(), e),
                    )
                })?;

                let end_x = match direction {
                    Directions::Right => x + distance,
                    Directions::Left => x - distance,
                    _ => x,
                };
                let end_y = match direction {
                    Directions::Up => y + distance,
                    Directions::Down => y - distance,
                    _ => y,
                };

                let segment = Segment {
                    length_before: length,
                    orientation: match direction {
                        Directions::Up | Directions::Down => Orientations::Vertical,
                        Directions::Left | Directions::Right => Orientations::Horizontal,
                    },
                    start: Coordinates { x, y },
                    end: Coordinates { x: end_x, y: end_y },
                };

                length += distance;
                x = end_x;
                y = end_y;

                Ok(segment)
            })
            .collect::<Result<Vec<Segment>, io::Error>>()?;

        wires.push(Wire { id, segments });
        id += 1;
    }

    Ok(wires)
}
