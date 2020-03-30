#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]

use std::cmp::{max, min};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::mpsc::{self, Receiver, Sender};

use intcode;

fn main() {
    let start_time = std::time::Instant::now();

    let program = intcode::load_program("day15/input.txt").unwrap_or_else(|err| {
        println!("Could not load input file!\n{:?}", err);
        std::process::exit(1);
    });

    let (in_send, in_recv) = mpsc::channel();
    let (out_send, out_recv) = mpsc::channel();
    let mut computer = intcode::Computer::new(&program, in_recv, out_send);
    std::thread::spawn(move || { computer.run(); });

    let mut droid = Droid::new(in_send, out_recv);
    let ship = droid.explore_ship();

    let distance_to_oxygen_system = search_maze(&ship.grid, ship.start, Some(ship.oxygen_system));
    let oxygen_distance = search_maze(&ship.grid, ship.oxygen_system, None);

    println!(
        "Part 1: {}\nPart 2: {}\nTime: {}ms",
        distance_to_oxygen_system,
        oxygen_distance,
        start_time.elapsed().as_millis()
    );
}

fn _print_ship(ship: &Ship) {
    for row in 0..ship.grid.len() {
        for col in 0..ship.grid[row].len() {
            if row == ship.start.row as usize && col == ship.start.col as usize {
                print!("D");
            } else if row == ship.oxygen_system.row as usize
                && col == ship.oxygen_system.col as usize
            {
                print!("O");
            } else {
                print!("{}", ship.grid[row][col]);
            }
        }
        println!();
    }
}

struct BFSState {
    pos: Position,
    distance: usize,
}

impl PartialEq for BFSState {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos
    }
}
impl Eq for BFSState {}

impl std::hash::Hash for BFSState {
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        self.pos.hash(state);
    }
}

// Perform a breadth-first search of a supplied maze from a given starting point.
// If `stop_at` is provided, returns the length of the shortest path to that point.
// Otherwise, returns the length of the longest shortest path to any point.
fn search_maze(grid: &[Vec<char>], from: Position, stop_at: Option<Position>) -> usize {
    let mut queue = VecDeque::new();
    let mut seen_states = HashSet::new();
    let mut max_distance = 0;
    queue.push_back(BFSState {
        pos: from,
        distance: 0,
    });

    while !queue.is_empty() {
        let state = queue.pop_front().unwrap();

        // Backtrack if we've been here before.
        if seen_states.contains(&state) {
            continue;
        }

        // Backtrack if we've walked into a wall.
        if grid[state.pos.row as usize][state.pos.col as usize] == '#' {
            continue;
        }

        if state.distance > max_distance {
            max_distance = state.distance;
        }

        // Stop if we've reached the target.
        if stop_at.is_some() && state.pos == stop_at.unwrap() {
            break;
        }

        // We're good, so add all adjacent positions to the queue.
        let adjacent_positions = [
            state.pos + &Direction::North,
            state.pos + &Direction::South,
            state.pos + &Direction::West,
            state.pos + &Direction::East,
        ];
        for pos in &adjacent_positions {
            queue.push_back(BFSState {
                pos: *pos,
                distance: state.distance + 1,
            });
        }

        // Remember that we've been here.
        seen_states.insert(state);
    }

    max_distance
}

#[derive(Clone, Copy)]
enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    fn turn_right(self) -> Self {
        match self {
            Self::North => Self::East,
            Self::East => Self::South,
            Self::South => Self::West,
            Self::West => Self::North,
        }
    }

    fn turn_left(self) -> Self {
        match self {
            Self::North => Self::West,
            Self::West => Self::South,
            Self::South => Self::East,
            Self::East => Self::North,
        }
    }

    fn input(self) -> i64 {
        match self {
            Self::North => 1,
            Self::South => 2,
            Self::West => 3,
            Self::East => 4,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Position {
    row: isize,
    col: isize,
}

impl Position {
    fn origin() -> Self {
        Self { row: 0, col: 0 }
    }

    fn extend_min(&mut self, other: &Self) {
        self.row = min(self.row, other.row);
        self.col = min(self.col, other.col);
    }

    fn extend_max(&mut self, other: &Self) {
        self.row = max(self.row, other.row);
        self.col = max(self.col, other.col);
    }
}

impl std::ops::Add for Position {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            row: self.row + other.row,
            col: self.col + other.col,
        }
    }
}

impl std::ops::Sub for Position {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            row: self.row - other.row,
            col: self.col - other.col,
        }
    }
}

impl std::ops::Add<&Direction> for Position {
    type Output = Self;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn add(self, other: &Direction) -> Self {
        match other {
            Direction::North => Self {
                row: self.row - 1,
                col: self.col,
            },
            Direction::South => Self {
                row: self.row + 1,
                col: self.col,
            },
            Direction::West => Self {
                row: self.row,
                col: self.col - 1,
            },
            Direction::East => Self {
                row: self.row,
                col: self.col + 1,
            },
        }
    }
}

enum MoveResult {
    Wall,
    Corridor,
    OxygenSystem,
}

impl MoveResult {
    fn from(output: i64) -> Self {
        match output {
            0 => Self::Wall,
            1 => Self::Corridor,
            2 => Self::OxygenSystem,
            _ => panic!("Unexpected output from computer: {}", output),
        }
    }
}

// A repository of information about a ship layout that we've learned so far.
struct LearnedShipInfo {
    // All the grid locations we've learned about.
    map: HashMap<Position, char>,

    // The location of the oxygen system.
    oxygen_system: Position,

    // The relative co-ordinates of the most top-left position we know about.
    min_pos: Position,

    // The relative co-ordinates of the most bottom-right position we know about.
    max_pos: Position,

    // Whether, in exploring this ship, we've moved away from the starting point.
    // We store this because we're finished when we return to the starting point.
    moved_off_starting_pos: bool,
}

impl LearnedShipInfo {
    fn new() -> Self {
        let mut info = Self {
            map: HashMap::new(),
            oxygen_system: Position::origin(),
            min_pos: Position::origin(),
            max_pos: Position::origin(),
            moved_off_starting_pos: false,
        };
        info.map.insert(Position::origin(), '.');
        info
    }
}

// Complete information about the ship layout.
struct Ship {
    // A visual representation of the ship layout, as a 2-D vector (row, then column).
    grid: Vec<Vec<char>>,

    // The droid's starting location in the ship.
    start: Position,

    // The location of the oxygen system.
    oxygen_system: Position,
}

impl Ship {
    fn construct(info: LearnedShipInfo) -> Self {
        let width = (info.max_pos.col - info.min_pos.col + 1) as usize;
        let height = (info.max_pos.row - info.min_pos.row + 1) as usize;
        let delta = Position::origin() - info.min_pos;

        let mut grid: Vec<Vec<char>> =
            std::iter::repeat(std::iter::repeat(' ').take(height).collect())
                .take(width)
                .collect();

        for (pos, c) in info.map {
            let adjusted_pos = pos + delta;
            grid[adjusted_pos.row as usize][adjusted_pos.col as usize] = c;
        }

        Self {
            grid,
            start: delta,
            oxygen_system: info.oxygen_system + delta,
        }
    }
}

// Our representation of the droid that's exploring the ship, with which we
// indirectly communicate via the Intcode computer.
struct Droid {
    // The droid's current location in the ship.
    pos: Position,

    // A channel to send instructions to the computer for moving the droid.
    tx: Sender<i64>,

    // A channel to receive movement results from the computer.
    rx: Receiver<i64>,
}

impl Droid {
    fn new(tx: Sender<i64>, rx: Receiver<i64>) -> Self {
        Self {
            pos: Position { row: 0, col: 0 },
            tx,
            rx,
        }
    }

    // Fully explore the ship.
    // Implements a basic "wall follower" algorithm, right-hand rule.
    fn explore_ship(&mut self) -> Ship {
        let mut learned_info = LearnedShipInfo::new();
        let mut dir = Direction::North;
        let origin = Position::origin();

        while !learned_info.moved_off_starting_pos || self.pos != origin {
            dir = self.attempt_move(&mut learned_info, dir);
        }

        Ship::construct(learned_info)
    }

    // Try to move in the specified direction, record what we learn by so doing,
    // and decide which direction to move in next. We simulate the droid keeping
    // its right hand on a wall at all times, so it tries to turn right if it
    // successfully moves forwards, and left if it hits a wall.
    fn attempt_move(&mut self, info: &mut LearnedShipInfo, dir: Direction) -> Direction {
        let target_pos = self.pos + &dir;

        self.tx.send(dir.input()).unwrap();
        let (map_char, new_dir, moved) = match MoveResult::from(self.rx.recv().unwrap()) {
            MoveResult::Wall => ('#', dir.turn_left(), false),
            MoveResult::Corridor => ('.', dir.turn_right(), true),
            MoveResult::OxygenSystem => {
                info.oxygen_system = target_pos;
                ('.', dir.turn_right(), true)
            }
        };

        info.min_pos.extend_min(&target_pos);
        info.max_pos.extend_max(&target_pos);

        if moved {
            self.pos = target_pos;
            info.moved_off_starting_pos = true;
        }

        info.map.insert(target_pos, map_char);

        new_dir
    }
}
