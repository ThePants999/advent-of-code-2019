#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]

use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;
use std::io::{self, Read};

#[macro_use]
extern crate lazy_static;
extern crate regex;
use regex::Regex;

const HEIGHT: usize = 150;
const WIDTH: usize = 150;
type Grid = [[char; WIDTH]; HEIGHT];
const DIRS_R: [isize; 4] = [0, 1, 0, -1];
const DIRS_C: [isize; 4] = [-1, 0, 1, 0];

// More noddy breadth-first search, so again not very performant (fortunately only a
// couple of seconds this time), and again you probably want to look at someone else's
// solution for this day.

fn main() {
    let start_time = std::time::Instant::now();

    let maze = load_maze().unwrap_or_else(|err| {
        println!("Could not load input file!\n{:?}", err);
        std::process::exit(1);
    });

    let part_1_answer = solve_maze(&maze, false).unwrap();
    let part_2_answer = solve_maze(&maze, true).unwrap();

    println!(
        "Part 1: {}\nPart 2: {}\nTime: {}ms",
        part_1_answer,
        part_2_answer,
        start_time.elapsed().as_millis()
    );
}

fn solve_maze(maze: &Maze, recursive: bool) -> Option<usize> {
    let mut queue = VecDeque::new();
    let mut seen_states = HashSet::new();

    queue.push_back(State {
        level: 0,
        location: maze.boundary.entrance,
        distance: 0,
    });

    while !queue.is_empty() {
        let mut state = queue.pop_front().unwrap();
        if seen_states.contains(&state) {
            continue;
        }

        if state.location == maze.boundary.exit {
            if state.level == 0 {
                return Some(state.distance - 1);
            } else {
                // Exit on inner levels is actually a wall
                continue;
            }
        }

        if let Some(portal_exit) = maze.portals.get(&state.location) {
            // Teleport!
            if recursive {
                if state.location.row == 1
                    || state.location.row == maze.bottom_row
                    || state.location.col == 1
                    || state.location.col == maze.rightmost_col
                {
                    // Outer - up a level
                    if state.level == 0 {
                        // Actually a wall
                        continue;
                    }
                    state.level -= 1;
                } else {
                    state.level += 1;
                }
            }

            state.location = *portal_exit;
        } else if maze.grid[state.location.row][state.location.col] == '#' {
            // Wall
            continue;
        }

        for dir in 0..4 {
            queue.push_back(State {
                level: state.level,
                location: Location {
                    row: (state.location.row as isize + DIRS_R[dir]) as usize,
                    col: (state.location.col as isize + DIRS_C[dir]) as usize,
                },
                distance: state.distance + 1,
            });
        }

        seen_states.insert(state);
    }

    None
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ParseState {
    AboveMaze,
    TopSection,
    InnerTop,
    InnerMiddle,
    InnerBottom,
    BottomSection,
    BelowMaze,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ParseSubState {
    LeftOfMaze,
    LeftSection,
    InnerSection,
    RightSection,
    RightOfMaze,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
struct Location {
    row: usize,
    col: usize,
}

struct PortalPair {
    a: Portal,
    b: Portal,
}

#[derive(Copy, Clone)]
struct Portal {
    entrance: Location,
    exit: Location,
}

#[derive(Clone)]
struct MazeBoundary {
    entrance: Location,
    exit: Location,
}

#[derive(Clone)]
struct Maze {
    grid: Grid,
    boundary: MazeBoundary,
    portals: HashMap<Location, Location>,
    bottom_row: usize,
    rightmost_col: usize,
}

#[derive(Clone)]
struct State {
    level: usize,
    location: Location,
    distance: usize,
}

impl PartialEq for State {
    fn eq(&self, other: &State) -> bool {
        self.location == other.location
    }
}
impl Eq for State {}

impl Hash for State {
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        self.location.row.hash(state);
        self.location.col.hash(state);
        self.level.hash(state);
    }
}

// All the code below this relates to constructing the maze from the input file - and it's
// horrible. I struggled with the portal names being two adjacent characters, where the
// adjacent character could be below or to the right, and which one was the actual portal
// location depended on where in the maze you were. If I did it again, I'd come up with
// something better, so I'm not going to explain this code - but nor am I going to improve
// it, sorry!
fn load_maze() -> Result<Maze, io::Error> {
    let mut input = std::fs::File::open("day20/input.txt")?;
    let mut input_maze = String::new();
    input.read_to_string(&mut input_maze)?;

    let mut grid: Grid = [['!'; WIDTH]; HEIGHT];
    let mut rows: Vec<String> = Vec::new();
    for (row, line) in input_maze.lines().enumerate() {
        rows.push(line.to_string());
        for (col, c) in line.chars().enumerate() {
            grid[row][col] = c;
        }
    }

    let mut map: HashMap<String, PortalPair> = HashMap::new();
    let mut boundary = MazeBoundary {
        entrance: Location { row: 0, col: 0 },
        exit: Location { row: 0, col: 0 },
    };
    let mut bottom_row = 0;
    let mut rightmost_col = 0;

    let mut state = ParseState::AboveMaze;
    let mut rows_in_current_state = 0;
    for row in 0..rows.len() {
        let (new_state, row_is_bottom_row) =
            check_state_transition(state, &mut rows_in_current_state, &rows[row]);
        state = new_state;
        if row_is_bottom_row {
            bottom_row = row;
        }

        if rows_in_current_state == 2
            && [
                ParseState::AboveMaze,
                ParseState::InnerBottom,
                ParseState::BelowMaze,
                ParseState::InnerTop,
            ]
            .contains(&state)
        {
            continue;
        }

        let mut substate = ParseSubState::LeftOfMaze;
        let mut skip_next_char = false;

        for col in 0..WIDTH {
            if skip_next_char {
                skip_next_char = false;
                continue;
            }

            if grid[row][col].is_alphabetic() {
                handle_portal(
                    &mut map,
                    &mut grid,
                    row,
                    col,
                    state,
                    substate,
                    &mut boundary,
                );

                if state == ParseState::TopSection
                    || state == ParseState::InnerMiddle
                    || state == ParseState::BottomSection
                {
                    skip_next_char = true;
                    substate = match substate {
                        ParseSubState::LeftOfMaze => ParseSubState::LeftSection,
                        ParseSubState::LeftSection => ParseSubState::InnerSection,
                        ParseSubState::InnerSection => ParseSubState::RightSection,
                        ParseSubState::RightSection => {
                            rightmost_col = col;
                            ParseSubState::RightOfMaze
                        }
                        ParseSubState::RightOfMaze => panic!("Unexpected RightOfMaze"),
                    };
                }
            } else if substate == ParseSubState::LeftOfMaze && grid[row][col] == '#' {
                substate = ParseSubState::LeftSection;
            } else if state == ParseState::InnerMiddle {
                if substate == ParseSubState::LeftSection && grid[row][col] == ' ' {
                    substate = ParseSubState::InnerSection;
                } else if substate == ParseSubState::InnerSection && grid[row][col] == '#' {
                    substate = ParseSubState::RightSection;
                } // Updating to RightOfMaze isn't necessary
            }
        }
    }

    let mut portals = HashMap::new();
    for pair in map.values() {
        portals.insert(pair.a.entrance, pair.b.exit);
        portals.insert(pair.b.entrance, pair.a.exit);
    }

    Ok(Maze {
        grid,
        boundary,
        portals,
        bottom_row,
        rightmost_col,
    })
}

fn parse_portal(
    grid: &Grid,
    row: usize,
    col: usize,
    state: ParseState,
    substate: ParseSubState,
) -> (String, Portal) {
    let mut portal_name = grid[row][col].to_string();
    let mut entrance = Location { row, col };
    let mut exit = Location { row, col };
    match state {
        ParseState::AboveMaze | ParseState::InnerBottom => {
            portal_name.push(grid[row + 1][col]);
            entrance.row += 1;
            exit.row += 2;
        }
        ParseState::BelowMaze | ParseState::InnerTop => {
            portal_name.push(grid[row + 1][col]);
            exit.row -= 1;
        }
        _ => {
            portal_name.push(grid[row][col + 1]);
            match substate {
                ParseSubState::LeftOfMaze | ParseSubState::InnerSection => {
                    entrance.col += 1;
                    exit.col += 2;
                }
                _ => {
                    exit.col -= 1;
                }
            }
        }
    }
    (portal_name, Portal { entrance, exit })
}

fn check_state_transition(
    state: ParseState,
    rows_in_current_state: &mut usize,
    current_row: &str,
) -> (ParseState, bool) {
    lazy_static! {
        static ref RE: Regex = Regex::new(r" [A-Z] ").unwrap();
    }

    let mut new_state = state;
    let mut row_is_bottom_row = false;

    *rows_in_current_state += 1;

    match state {
        ParseState::AboveMaze => {
            if *rows_in_current_state == 3 {
                new_state = ParseState::TopSection;
            }
        }
        ParseState::TopSection => {
            if current_row.contains("#   ") {
                new_state = ParseState::InnerTop;
            }
        }
        ParseState::InnerTop => {
            if *rows_in_current_state == 3 {
                new_state = ParseState::InnerMiddle;
            }
        }
        ParseState::InnerMiddle => {
            if RE.is_match(current_row) {
                new_state = ParseState::InnerBottom;
            }
        }
        ParseState::InnerBottom => {
            if *rows_in_current_state == 3 {
                new_state = ParseState::BottomSection;
            }
        }
        ParseState::BottomSection => {
            if current_row.contains("   ") {
                new_state = ParseState::BelowMaze;
                row_is_bottom_row = true;
            }
        }
        ParseState::BelowMaze => (),
    }

    if new_state != state {
        *rows_in_current_state = 1;
    }

    (new_state, row_is_bottom_row)
}

fn handle_portal(
    map: &mut HashMap<String, PortalPair>,
    grid: &mut Grid,
    row: usize,
    col: usize,
    state: ParseState,
    substate: ParseSubState,
    boundary: &mut MazeBoundary,
) {
    let (name, portal) = parse_portal(&grid, row, col, state, substate);
    if name.eq("AA") {
        boundary.entrance.row = portal.exit.row;
        boundary.entrance.col = portal.exit.col;
        // Don't let the seeker exit the maze via the entrance
        grid[portal.entrance.row][portal.entrance.col] = '#';
    } else if name.eq("ZZ") {
        boundary.exit.row = portal.entrance.row;
        boundary.exit.col = portal.entrance.col;
    } else if let Some(pair) = map.get_mut(&name) {
        // Update existing portal pair
        pair.b = portal;
    } else {
        map.insert(
            name.clone(),
            PortalPair {
                a: portal,
                b: Portal {
                    entrance: Location { row: 0, col: 0 },
                    exit: Location { row: 0, col: 0 },
                },
            },
        );
    }
}
