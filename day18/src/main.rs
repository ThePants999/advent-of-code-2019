#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]

use std::fmt;

use std::collections::{HashSet, VecDeque};
use std::hash::{Hash, Hasher};

const HEIGHT: usize = 81;
const WIDTH: usize = 81;
type Maze = [[char; HEIGHT]; WIDTH];
const DIRS_X: [isize; 4] = [0, 1, 0, -1];
const DIRS_Y: [isize; 4] = [-1, 0, 1, 0];

const INPUT: &str = 
"#################################################################################
#.....#.#...#...........Z...........#...#.......#...#...........#.........#.....#
#.###.#.#.#.#.###########.#########.###.#.#####.#.#.#.#.#######.#####.###.#.###.#
#.#.....#.#.#.#b#.......#...#...#.#...#.#.#...#...#.#.#...#...#...#...#.#.#...#.#
#M#######.#.#.#.#.###.#####.#.#.#.###.#.#.###.#####.#.###.#.#.###.#.###.#.###.#.#
#.........#.#.#.#...#.....#.#.#.....#.#.#...#.....#.....#.#.#...#...#.....#...#.#
#.#########.#.#.###.#####.#.#.#####.#.#.###.#.###.#######.#.###.#####.#####.###.#
#...#.....#...#.....#.....#.#.....#.#...#.#.#.#.....#...#.#.#.#.....#.....#.#.#.#
###.###.#.###########.###.#.#####.#####.#.#.#.#######.#.#.#.#.###.#######.#.#.#.#
#...#.T.#.#...........#...#.....#.......#.#.#...#.....#...#.#...#.......#...#.#.#
#.###.###.#.###########.#######.###.#####.#.###.#.#########.###.#######.#####.#.#
#.....#.#.#.#.........#.#.....#...#.#...#.......#...#.....#.#.........#i....#...#
#######.#.#.#.#######.#.#.###.###.#.#.#.###########.#.###.#.#.#######.###.###.###
#.#.....#.#.#...#.....#.#.#.#.#...#.#.#.#.........#.#.#.#...#.....#.#.#...#...#.#
#.#.#.###.#V###.#.#######.#.#.#.###.#.#.#.#######.#.#.#.#########.#.#.#.###.###.#
#...#.......#.#.#.....#...#...#.#...#.#.#a#.........#.........#...#.#.#.....#...#
###########.#.#.#####.#.###.###.#.#####.#.###################.#.###.#.#######.###
#.....#...#.#.#.#...#.#.#.#...#.#...#...#.#.....#...........#.#...#.#.#........e#
#.###.#.#.#.#.#.#.#.#.#.#.###.#.###.#.#.#.#.###.#######.#.###.###.#.#.#########.#
#...#...#.#...#.#.#.....#...#.#.#.....#.#.#.#.#.......#.#.........#.....#.......#
###.#####.###.#.#########.#.#.#.#######.#.#.#.#######.#.###############.#.#######
#.#.#.....#...#.....#.#...#.#.#.......#.#.#...#...#...#.#...#.#.....#..w#.#.....#
#.#.#.#############.#.#.###.#.#####.#.###Q###.#.#.#.###.###.#.#.###.#.###.#.###.#
#.#.#.............#...#.#...#.....#.#...#.....#.#...#.......#.#.#...#.....#.#.#.#
#.#.#############.#####.#.#######.#.###.#.#####.###.#######.#.#.###.###.###.#.#.#
#.#.#.........#.#.......#.#.......#...#.#.#...#.#...#.....#.#.#...#...#.#...#.#.#
#.#.#.#.#####.#.###.#####.#.###########.#.###.#.#####.###.#.#.#.#.###.###.###.#.#
#.#.#.#.#x..#...#.#.#...#.#...#.........#.....#.#.....#.#.#.#.#.#...#s..#.#.#...#
#.#.#.###.#.###.#.#.#.#.#####.#.#.#.#####.#####.#.#####.#.###.#####.###.#.#.#.###
#.#.#...#.#...#...#...#.......#.#.#.#...#.#.....#.....#.#...#.......#.#...#.#...#
#.#.#.#.#.###.###################.###.#.#.#.###.#####.#.###.#########.#####.###.#
#...#.#...#.#.............#.....#...#.#.#.#.#.....#...#...#...#.......#.....#...#
#.###.#####.#######.#####.###.#.#.#.#.#.#.#.#.#####.###.#.#.###.#.###.#.###.#.###
#...#.......#...#...#.#.C.#...#.#.#...#.#.#.#.#...#...#.#.#.....#.#...#.#...#.#.#
###.#######.###.#.###.#.###.###.#######.#.#.###.#.###.#.#.#######.#.###.#####.#.#
#.#...#.....#...#.#.....#.....#...#...#.#.#...#.#.....#.#...#...#.#.#...#...#...#
#.###.#.#####.#.#.###.#######.###.#.#.#.#.###.#.#######.###.#.#.#.#.#.###.#.###.#
#...#.#.......#.#...#...#...#.#...#.#...#...#.#.#...#...#.#.#.#.#.#...#...#...#.#
#.#.#.#############.###.#.#.#.#.###.###.###.#.#.#.#.###.#.#.###.#.#####.###.###.#
#.#.................#.....#...#.....#..@#@..#.....#.....#.......#.........#.....#
#################################################################################
#h#.......#.#...........#.......#.#....@#@..........#.......#......c....#.#.....#
#.#.###.#.#.#.#####.###.#.#####F#.#.#.#####.###.#####.#.#####.#.#######.#.#.#.#.#
#.#...#.#...#l..J.#.#.L.#.#.#...#...#...#...#...#.....#.......#.#.....#...#.#.#.#
#O###.#.#########.#.#####.#.#.#########.#.#######.#############.#.###.#####.#.###
#.#...#....j..#...#.....#f..#.#.........#q#.....#...#...#.....#.#.#.#.......#...#
#.#.#######.###.#######.#.###.#.#######.#.#.###.###.#.#.#####.#.#.#.#########.#.#
#.#.#.....#...#.#.....#...#..k#.#.....#.#...#.#.....#.#.....#.#...#.#...#...#.#.#
#.#.#.###.###.#.###.#######.###.###.#.#.#.###.#######.#####.#.#####.#.#.#.#.###.#
#.#.#...#...#...#...#.......#.....#.#...#.....#.......#.......#.....#.#...#.#...#
#.#.###.###.#####.#.#K###########.###.#######.#.#####.#######.###.###.#####.#.###
#.#.#...#.#...#...#.#....d#.....#...#...#...#.#.#...#.....#.#.#...#...#...#.#...#
#.#.#.###.###.###.#######.#.###D###.#####.#.#.###.#.#.###.#.#.#.#.#.#####.#.###.#
#...#...#...#...#.#.....#.....#...#.....#.#.#.....#.#...#.#..u..#.#.#...#.....X.#
#.###.#.#.#####.#.#.###P###.#####.#####.#.#########.###.#.#########.#.#.#######.#
#.#.E.#.#.........#g#.#...#.#...#...#...#.........#.#...#.#.......#...#...#v..#.#
#.#####.#H#########.#.###.#.#.#.###.#.###.#.#######.#.###.#.#####.#.#####.#.#.###
#..o..#.#.#.....#...#p....#.#.#..y..#...#.#.#.....#.#...#.#.....#.#.#...#...#...#
#####.#.###.###.#.###.#######.#########.###.#.###.#.#####.#####.#.#.#.#.#######.#
#.....#...#...#.#...#.......Y.#.......#.#...#...#.#...#...#.#...#.#.#.#.....#...#
#.#######.#.###.###.###########.#####.#.#.#####.#.###.#.#.#.#.###.#.#.#####.#.###
#...#...#...#.....#.#...#...........#...#.#.....#.#...#.#...#.#.#.#.#.....#...#.#
###.#.#.#####.###.#.###.###########.###.#.#.#####.#.###.###.###.#.#.#####.#####.#
#.#...#.W.....#...#...#.#.........#...#.#...#.....#.#.#...#...B.#.#.#.#...#.....#
#.#############.###G###N#.#######.#.#.#.#####.#####A#.#.#.#####.#.#.#.#.###.#####
#.....R...#.U.#.#...#...#.#..m..#.#.#.#.#.....#.....#.#.#...#...#.#...#..r#.....#
#.###.#####.#.#.#.###.###.#.###.#.#.#.###.###.#.#####.#.###.###.#.#######.#.###.#
#.#...#...#.#...#..n..#...#...#.#.#z#...#.#.#.#.....#.....#...#.#.......#.#.#...#
###.###.#.#.#############.#.#.###.#####.#.#.#.#####.#########.#.#######.#.#.#.###
#...#...#...#.......#.....#.#...#.......#.#.......#.......#...#.....#.#...#.#...#
#.###.#######.#####.#.#########.#######.#.#######.#######.#.#######.#.#####.###.#
#...#.#.....#.....#.#...#.....#...#...#.#...#.....#.....#.#.......#.....#...#...#
#.#.#S#.###.#####.#.#.#.#.###.###.#.#.#.#.#.#.#######.###.#.#####.#####.#.###.#.#
#.#.#.#.#.#.......#.#.#...#.#...#...#...#.#.#...#.....#...#.....#.....#.#.#...#.#
###.#.#.#.#########.#.#####.###.#.#########.###.###.###.#######.#####.###.#.###.#
#.I.#.#.#.....#...#.#.........#.#...#...#...#.#.....#...#...#...#...#.....#.#.#.#
#.#.#.#.#.#.###.#.#.###########.###.###.#.###.#.#####.###.#.###.#.#.#######.#.#.#
#.#.#...#.#.....#...#.........#...#.....#.#.....#.....#...#...#.#.#.....#...#...#
#.#######.###########.#######.###.#####.#.#######.#####.#####.###.#####.#.###.###
#.....................#...........#.....#...............#.........#....t..#.....#
#################################################################################";

fn main() {
    let input = String::from(INPUT);

    let mut maze: Maze = [['!'; HEIGHT]; WIDTH];
    let mut all_keys = HashSet::new();
    let mut starting_locations = Vec::with_capacity(4);

    for (y, line) in input.lines().enumerate() {
        for (x, c) in line.chars().enumerate() {
            maze[x][y] = match c {
                '@' => {
                    starting_locations.push(Location{ x, y });
                    '.'
                },
                door if door.is_uppercase() => {
                    door
                },
                key if key.is_lowercase() => {
                    all_keys.insert(key);
                    key
                },
                '#' => '#',
                '.' => '.',
                _ => panic!("Unexpected char in input"),
            };
        }
    }

    let mut queue = VecDeque::new();
    let mut seen_states = HashSet::new();
    for i in 0..starting_locations.len() {
        let this_location = starting_locations[i];
        let mut other_locations = starting_locations.clone();
        other_locations.remove(i);
        other_locations.shrink_to_fit();
        queue.push_back(State {
            location: this_location,
            keys: String::new(),
            distance: 0,
            other_robots: other_locations,
        });
    }

    while !queue.is_empty() {
        let state = queue.pop_front().unwrap();
        if seen_states.contains(&state) { continue; }

        let location = maze[state.location.x][state.location.y];

        // Don't bother with bounds check - the mazes all have walls at the edges
        if location == '#' {
            // Wall
            continue;
        }
        if location.is_uppercase() && !state.keys.contains(location.to_ascii_lowercase().to_string().as_str()) { 
            // Locked door
            continue;
        }
        let mut keys = state.keys.clone();
        //let mut keys_str = state.keys_str.clone();
        if location.is_lowercase() && !keys.contains(location.to_string().as_str()) {
            // Key
            let mut keys_vec: Vec<char> = keys.chars().collect();
            keys_vec.push(location);
            if keys_vec.len() == all_keys.len() {
                println!("{}", state.distance);
                break;
            }
            keys_vec.sort();
            keys = keys_vec.iter().collect();

            // Release the other robots!
            for i in 0..state.other_robots.len() {
                let mut other_robots = state.other_robots.clone();
                let other_loc = other_robots.remove(i);
                other_robots.push(state.location);
                queue.push_back(State {
                    location: other_loc,
                    keys: keys.clone(),
                    distance: state.distance,
                    other_robots,
                });
            }
        }

        for dir in 0..4 {
            let (x, y) = ((state.location.x as isize + DIRS_X[dir]) as usize, (state.location.y as isize + DIRS_Y[dir]) as usize);
            if maze[x][y] != '#' {
                queue.push_back(State {
                    location: Location {
                        x,
                        y,
                    },
                    keys: keys.clone(),
                    distance: state.distance + 1,
                    other_robots: state.other_robots.clone(),
                });
            }
        }

        seen_states.insert(state);

        if seen_states.len() % 100_000 == 0 { println!("{} - {}", seen_states.len(), queue.len()); }
    }

    // for node in nodes.values_mut() {
    //     node.find_edges(&maze);
    // }

    // let starting_node = nodes.get(&'@').unwrap();
    // println!("{}\nCurrently-valid edges:", starting_node);
    // for edge in starting_node.valid_edges(&doors) {
    //     println!("{}", edge);
    // }

    // let route = search(starting_node, Route::new(), &nodes, &doors, SearchModes::Naive).unwrap();
    // let route = search(starting_node, Route::new(), &nodes, &doors, SearchModes::Full(route.length)).unwrap();
    // println!("Route: {}\nLength: {}", route.route, route.length);

}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Location {
    x: usize,
    y: usize,
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl fmt::Debug for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[derive(Clone)]
struct State {
    location: Location,
    keys: String,
    distance: usize,
    other_robots: Vec<Location>,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {:?}", self.location, self.keys, self.other_robots)
    }
}

impl PartialEq for State {
    fn eq(&self, other: &State) -> bool {
        self.location == other.location && self.keys == other.keys && self.other_robots == other.other_robots
    }
}
impl Eq for State {}

impl Hash for State {
    fn hash<H>(&self, state: &mut H)
    where H: Hasher {
        self.location.x.hash(state);
        self.location.y.hash(state);
        self.keys.hash(state);
        for loc in &self.other_robots {
            loc.x.hash(state);
            loc.y.hash(state);
        }
    }
}

// fn search(from: &Node, route_to_here: Route, nodes: &HashMap<char, Node>, doors: &HashSet<char>, mode: SearchModes) -> Option<Route> {
//     println!("Trying {}", route_to_here.route);
//     let mut nodes = nodes.clone();
//     nodes.remove(&from.name);

//     if nodes.is_empty() {
//         // We're done.
//         return Some(route_to_here);
//     }

//     let mut doors = doors.clone();
//     doors.remove(&from.name.to_ascii_uppercase());

//     let mut routes = Vec::new();
//     let mut edges_to_try = from.valid_edges(&doors);
//     for edge in edges_to_try {
//         if let Some(target) = nodes.get(&edge.destination) {
//             let mut route = route_to_here.route.clone();
//             route.push(target.name);
//             let route_to_there = Route {
//                 route: route,
//                 length: route_to_here.length + edge.length,
//             };
//             if let SearchModes::Full(max_len) = mode {
//                 if route_to_there.length > max_len { continue; }                
//             }
//             if let Some(route) = search(target, route_to_there, &nodes, &doors, mode) {
//                 routes.push(route);
//                 if mode == SearchModes::Naive { break; }
//             }
//         }
//     }

//     if routes.is_empty() {
//         // No valid routes from here finish the job.
//         return None;
//     }

//     let mut best_route = routes[0].clone();
//     for route in routes {
//         if route.length < best_route.length { best_route = route.clone(); }
//     }

//     Some(best_route)
// }

// #[derive(PartialEq, Eq, Clone, Copy)]
// enum SearchModes {
//     Naive,
//     Full(usize),
// }

// #[derive(Clone)]
// struct Route {
//     route: String,
//     length: usize,
// }

// impl Route {
//     fn new() -> Self {
//         Self {
//             route: String::new(),
//             length: 0,
//         }
//     }
// }

// #[derive(Clone)]
// struct Edge {
//     length: usize,
//     destination: char,
//     blocked_by: Vec<char>,
// }

// impl fmt::Display for Edge {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "{} - {} steps, blocked by ({})", self.destination, self.length, self.blocked_by.iter().collect::<String>())
//     }
// }

// #[derive(Clone)]
// struct Node {
//     name: char,
//     x: usize,
//     y: usize,
//     edges: Vec<Edge>,
// }

// impl Node {
//     fn new (name: char, x: usize, y: usize) -> Self {
//         Self {
//             name: name,
//             x: x,
//             y: y,
//             edges: Vec::new(),
//         }
//     }

//     fn find_edges(&mut self, original_maze: &Maze) {
//         let mut maze = original_maze.clone();
//         maze[self.x][self.y] = '#';
    
//         let mut heads = Vec::new();
//         heads.push(SearchHead::new(self));
//         while !heads.is_empty() {
//             let mut new_heads = Vec::new();
//             for head in heads {
//                 new_heads.append(&mut head.search(&mut maze, self));
//             }
//             heads = new_heads;
//         }
//     }

//     fn valid_edges(&self, doors: &HashSet<char>) -> Vec<&Edge> {
//         self.edges.iter().filter(|edge| {
//             let mut edge_unblocked = true;
//             for door in &edge.blocked_by {
//                 if doors.contains(&door) {
//                     edge_unblocked = false;
//                     break;
//                 }
//             }
//             edge_unblocked
//         }).collect()
//     }
// }

// impl fmt::Display for Node {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         let mut edges = String::new();
//         for edge in &self.edges { edges.push_str(&format!("\n{}", edge)); }
//         write!(f, "-----Node {} ({}, {})-----{}", self.name, self.x, self.y, edges)
//     }
// }

// struct SearchHead {
//     distance: usize,
//     x: usize,
//     y: usize,
//     doors: Vec<char>,
// }

// impl SearchHead {
//     fn new(node: &Node) -> SearchHead {
//         SearchHead {
//             distance: 0,
//             x: node.x,
//             y: node.y,
//             doors: Vec::new(),
//         }
//     }

//     fn search(self, maze: &mut Maze, origin: &mut Node) -> Vec<SearchHead> {
//         let mut new_heads = Vec::new();

//         if self.x > 0 { if let Some(head) = self.try_spread_to(maze, origin, self.x - 1, self.y) { new_heads.push(head); } }
//         if self.x < WIDTH - 1 { if let Some(head) = self.try_spread_to(maze, origin, self.x + 1, self.y) { new_heads.push(head); } }
//         if self.y > 0 { if let Some(head) = self.try_spread_to(maze, origin, self.x, self.y - 1) { new_heads.push(head); } }
//         if self.y < HEIGHT - 1 { if let Some(head) = self.try_spread_to(maze, origin, self.x, self.y + 1) { new_heads.push(head); } }
//         new_heads
//     }

//     fn try_spread_to<'b>(&self, maze: &mut Maze, origin: &mut Node, x: usize, y: usize) -> Option<SearchHead> {
//         match maze[x][y] {
//             '#' => None,
//             '.' => { Some(self.spread_to(maze, x, y)) },
//             door if door.is_uppercase() => {
//                 let mut new_head = self.spread_to(maze, x, y);
//                 new_head.doors.push(door);
//                 Some(new_head)
//             },
//             key => {
//                 let edge = Edge {
//                     destination: key,
//                     length: self.distance + 1,
//                     blocked_by: self.doors.clone(),
//                 };
//                 origin.edges.push(edge);
//                 Some(self.spread_to(maze, x, y))
//             }
//         }
//     }

//     fn spread_to(&self, maze: &mut Maze, x: usize, y: usize) -> SearchHead {
//         maze[x][y] = '#';
//         SearchHead {
//             distance: self.distance + 1,
//             x: x,
//             y: y,
//             doors: self.doors.clone(),
//         }    
//     }
// }