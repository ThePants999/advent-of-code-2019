use std::io::{self, Read};
use std::fs::File;
use std::process;

use std::fmt;

use std::collections::{HashSet, VecDeque, HashMap};
use std::hash::{Hash, Hasher};

extern crate regex;
use regex::Regex;

const HEIGHT: usize = 150;
const WIDTH: usize = 150;
type Grid = [[char; WIDTH]; HEIGHT];
const DIRS_R: [isize; 4] = [0, 1, 0, -1];
const DIRS_C: [isize; 4] = [-1, 0, 1, 0];

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

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(row {}, col {})", self.row + 1, self.col + 1)
    }
}

impl fmt::Debug for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(row {}, col {})", self.row + 1, self.col + 1)
    }
}

#[derive(Debug)]
struct PortalPair {
    name: String,
    a: Portal,
    b: Portal,
}

#[derive(Copy, Clone, Debug)]
struct Portal {
    entrance: Location,
    exit: Location,
}

struct Maze {
    grid: Grid,
    entrance: Location,
    exit: Location,
    portals: HashMap<Location, Location>,
    bottom_row: usize,
    rightmost_col: usize,
}

fn load_maze() -> Result<Maze, io::Error> {
    let mut input = File::open("day20/input.txt")?;
    let mut input_maze = String::new();
    input.read_to_string(&mut input_maze)?;

    let mut grid: Grid = [['!'; WIDTH];HEIGHT];
    let mut rows: Vec<String> = Vec::new();
    for (row, line) in input_maze.lines().enumerate() {
        rows.push(line.to_string());
        for (col, c) in line.chars().enumerate() {
            grid[row][col] = c;
        }
    }

    let mut map: HashMap<String, PortalPair> = HashMap::new();
    let mut entrance_row = 0;
    let mut entrance_col = 0;
    let mut exit_row = 0;
    let mut exit_col = 0;
    let mut bottom_row = 0;
    let mut right_col = 0;

    let mut state = ParseState::AboveMaze;
    let mut rows_in_current_state = 0;
    let re = Regex::new(r" [A-Z] ").unwrap();
    for row in 0..rows.len() {
        rows_in_current_state += 1;
        let old_state = state;

        // Handle state transitions
        match state {
            ParseState::AboveMaze => {
                if rows_in_current_state == 3 { state = ParseState::TopSection; }
            },
            ParseState::TopSection => {
                if rows[row].contains("#   ") { state = ParseState::InnerTop; }
            },
            ParseState::InnerTop => {
                if rows_in_current_state == 3 { state = ParseState::InnerMiddle; }
            },
            ParseState::InnerMiddle => {
                if re.is_match(&rows[row]) { state = ParseState::InnerBottom; }
            },
            ParseState::InnerBottom => {
                if rows_in_current_state == 3 { state = ParseState::BottomSection; }
            },
            ParseState::BottomSection => {
                if rows[row].contains("   ") {
                    state = ParseState::BelowMaze; 
                    bottom_row = row;
                }
            },
            ParseState::BelowMaze => (),
        }
        if state != old_state { rows_in_current_state = 1; }

        if (state == ParseState::AboveMaze || state == ParseState::InnerBottom) && rows_in_current_state == 2 {
            continue;
        } else if (state == ParseState::BelowMaze || state == ParseState::InnerTop) && rows_in_current_state == 2 {
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
                let (name, portal) = parse_portal(&grid, row, col, state, substate);
                if name.eq("AA") {
                    entrance_row = portal.exit.row;
                    entrance_col = portal.exit.col;
                    // Don't let the seeker exit the maze via the entrance
                    grid[portal.entrance.row][portal.entrance.col] = '#';
                } else if name.eq("ZZ") {
                    exit_row = portal.entrance.row;
                    exit_col = portal.entrance.col;
                } else if let Some(pair) = map.get_mut(&name) {
                    // Update existing portal pair
                    pair.b = portal;
                } else {
                    map.insert(name.clone(), PortalPair {
                        name: name,
                        a: portal,
                        b: Portal { entrance: Location { row: 0, col: 0 }, exit: Location { row: 0, col: 0 }},
                    });
                }    

                if state == ParseState::TopSection || state == ParseState::InnerMiddle || state == ParseState::BottomSection {
                    skip_next_char = true;
                    substate = match substate {
                        ParseSubState::LeftOfMaze => ParseSubState::LeftSection,
                        ParseSubState::LeftSection => ParseSubState::InnerSection,
                        ParseSubState::InnerSection => ParseSubState::RightSection,
                        ParseSubState::RightSection => {
                            right_col = col;
                            ParseSubState::RightOfMaze
                        },
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
        grid: grid,
        entrance: Location { row: entrance_row, col: entrance_col, },
        exit: Location { row: exit_row, col: exit_col, },
        portals: portals,
        bottom_row: bottom_row,
        rightmost_col: right_col,
    })
}

fn parse_portal(grid: &Grid, row: usize, col: usize, state: ParseState, substate: ParseSubState) -> (String, Portal) {
    let mut portal_name = grid[row][col].to_string();
    let mut entrance = Location { row: row, col: col };
    let mut exit = Location { row: row, col: col };
    match state {
        ParseState::AboveMaze | ParseState::InnerBottom => {
            portal_name.push(grid[row + 1][col]);
            entrance.row += 1;
            exit.row += 2;
        },
        ParseState::BelowMaze | ParseState::InnerTop => {
            portal_name.push(grid[row + 1][col]);
            exit.row -= 1;
        },
        _ => {
            portal_name.push(grid[row][col + 1]);
            match substate {
                ParseSubState::LeftOfMaze | ParseSubState::InnerSection => {
                    entrance.col += 1;
                    exit.col += 2;
                },
                _ => { exit.col -= 1; }
            }
        }
    }
    (portal_name, Portal { entrance: entrance, exit: exit })
}

fn main() {
    let maze = load_maze().unwrap_or_else(|err| {
        println!("Could not load input file!\n{:?}", err);
        process::exit(1);
    });

    //let mut out_file = File::create("day20/output.txt").unwrap();

    let mut queue = VecDeque::new();
    let mut seen_states = HashSet::new();

    queue.push_back(State {
        level: 0,
        location: maze.entrance,
        distance: 0,
    });

    while !queue.is_empty() {
        let mut state = queue.pop_front().unwrap();
        if seen_states.contains(&state) { continue; }

        if state.location == maze.exit {
            if state.level == 0 {
                println!("{}", state.distance - 1);
                break;
            } else {
                // Exit on inner levels is actually a wall
                continue;
            }
        }

        if let Some(portal_exit) = maze.portals.get(&state.location) {
            // Teleport!
            if state.location.row == 1 || state.location.row == maze.bottom_row || state.location.col == 1 || state.location.col == maze.rightmost_col {
                // Outer - up a level
                if state.level == 0 {
                    // Actually a wall
                    continue;
                }
                state.level -= 1;
            } else {
                state.level += 1;
            }

            //out_file.write_all(format!("Teleported from {} to {} on level {} ({})\n", state.location, portal_exit, state.level, state.distance).as_bytes()).unwrap();
            state.location = *portal_exit;
        } else if maze.grid[state.location.row][state.location.col] == '#' {
            // Wall
            continue;
        } else {
            //out_file.write_all(format!("Moved to {}:{} ({})\n", state.level, state.location, state.distance).as_bytes()).unwrap();
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

        if seen_states.len() % 100000 == 0 { println!("{} - {}", seen_states.len(), queue.len()); }
    }
}

#[derive(Clone)]
struct State {
    level: usize,
    location: Location,
    distance: usize,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} after {} steps", self.location, self.distance)
    }
}

impl PartialEq for State {
    fn eq(&self, other: &State) -> bool {
        self.location == other.location
    }
}
impl Eq for State {}

impl Hash for State {
    fn hash<H>(&self, state: &mut H)
    where H: Hasher {
        self.location.row.hash(state);
        self.location.col.hash(state);
        self.level.hash(state);
    }
}