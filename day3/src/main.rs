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
                    // Of course they intersect at the origin - ignore that!
                    continue;
                }
                manhattan_distances.push(int.manhattan_distance);
                wire_lengths.push(int.wire_length);
            }
        }
    }

    println!(
        "Shortest Manhattan: {}\nShortest length: {}\nTime: {}us",
        manhattan_distances.iter().min().unwrap(),
        wire_lengths.iter().min().unwrap(),
        start_time.elapsed().as_micros()
    );
}

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn end(&self, start: Coordinates, distance: i32) -> Coordinates {
        match self {
            Self::Up => Coordinates { x: start.x, y: start.y + distance },
            Self::Down => Coordinates { x: start.x, y: start.y - distance },
            Self::Right => Coordinates { x: start.x + distance, y: start.y },
            Self::Left => Coordinates { x: start.x - distance, y: start.y },
        }
    }

    fn orientation(&self) -> Orientation {
        match self {
            Self::Up | Self::Down => Orientation::Vertical,
            Self::Left | Self::Right => Orientation::Horizontal,
        }
    }
}

#[derive(PartialEq, Eq)]
enum Orientation {
    Horizontal,
    Vertical,
}

#[derive(PartialEq, Eq, Copy, Clone)]
struct Coordinates {
    x: i32,
    y: i32,
}

impl Coordinates {
    fn origin() -> Self {
        Self { x: 0, y: 0 }
    }
}

// A single piece of a wire.
struct Segment {
    length_before: i32,
    orientation: Orientation,
    start: Coordinates,
    end: Coordinates,
}

impl Segment {
    // Determine whether this segment intersects another one, and if so, where.
    fn intersects(&self, other: &Segment) -> Option<Coordinates> {
        // I'm going to be cheeky and ignore the possibility of two horizontal or two
        // vertical segments intersecting along part of their length.
        //
        // Also, yuuuuck.
        if self.orientation == other.orientation {
            if (self.start == other.start) || (self.start == other.end) {
                Some(self.start)
            } else if (self.end == other.start) || (self.end == other.end) {
                Some(self.end)
            } else {
                None
            }
        } else if self.orientation == Orientation::Horizontal {
            if (cmp::min(self.start.x, self.end.x) <= other.start.x)
                && (cmp::max(self.start.x, self.end.x) >= other.start.x)
                && (cmp::min(other.start.y, other.end.y) <= self.start.y)
                && (cmp::max(other.start.y, other.end.y) >= self.start.y)
            {
                Some(Coordinates {
                    x: other.start.x,
                    y: self.start.y,
                })
            } else {
                None
            }
        } else if (cmp::min(self.start.y, self.end.y) <= other.start.y)
            && (cmp::max(self.start.y, self.end.y) >= other.start.y)
            && (cmp::min(other.start.x, other.end.x) <= self.start.x)
            && (cmp::max(other.start.x, other.end.x) >= self.start.x)
        {
            Some(Coordinates {
                x: self.start.x,
                y: other.start.y,
            })
        } else {
            None
        }
    }
}

struct Wire {
    segments: Vec<Segment>,
}

struct Intersection {
    coords: Coordinates,
    manhattan_distance: i32,
    wire_length: i32,
}

fn calculate_intersection(first: &Segment, second: &Segment) -> Option<Intersection> {
    first.intersects(second).map(|coords| {
        let first_length = manhattan_distance(coords, first.start) + first.length_before;
        let second_length = manhattan_distance(coords, second.start) + second.length_before;
        Intersection {
            coords,
            manhattan_distance: manhattan_distance(coords, Coordinates { x: 0, y: 0 }),
            wire_length: first_length + second_length,
        }
    })
}

fn manhattan_distance(first: Coordinates, second: Coordinates) -> i32 {
    (second.x - first.x).abs() + (second.y - first.y).abs()
}

// Load the set of wires from file.
fn load_wires() -> Result<Vec<Wire>, io::Error> {
    let mut input_file = std::fs::File::open("day3/input.txt")?;
    let mut input = String::new();
    input_file.read_to_string(&mut input)?;

    let mut wires: Vec<Wire> = Vec::new();
    for line in input.lines() {
        let wire = read_wire(line)?;
        wires.push(wire);
    }

    Ok(wires)
}

// Construct a complete wire from a line of the input file.
fn read_wire(input_line: &str) -> Result<Wire, io::Error> {
    let mut length = 0;
    let mut pos = Coordinates::origin();
    let segments = input_line
        .split(',')
        .map(|cmd_text| { build_segment(&mut pos, &mut length, cmd_text) })
        .collect::<Result<Vec<Segment>, io::Error>>()?;

    Ok(Wire { segments })
}

// Construct a wire segment from a section of a wire line. Repeated calls
// keep track of start position and wire length.
fn build_segment(start: &mut Coordinates, length: &mut i32, command: &str) -> Result<Segment, io::Error> {
    let mut chars = command.chars();
    let direction = parse_direction(chars.next())?;
    let distance = chars.as_str().parse::<i32>().map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Invalid distance: {} ({})", chars.as_str(), e),
        )
    })?;
    let end = direction.end(*start, distance);

    let segment = Segment {
        length_before: *length,
        orientation: direction.orientation(),
        start: *start,
        end,
    };

    *length += distance;
    *start = end;

    Ok(segment)
}

// Read the first character of a wire segment to determine its direction.
fn parse_direction(c: Option<char>) -> Result<Direction, io::Error> {
    match c {
        Some('U') => Ok(Direction::Up),
        Some('D') => Ok(Direction::Down),
        Some('L') => Ok(Direction::Left),
        Some('R') => Ok(Direction::Right),
        Some(invalid_char) => {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Invalid direction: {}", invalid_char),
            ))
        }
        None => {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                String::from("Empty command"),
            ))
        }
    }
}