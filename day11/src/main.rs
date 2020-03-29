use std::collections::HashMap;
use std::process;
use std::sync::mpsc;

use intcode;
#[macro_use]
extern crate itertools;

fn main() {
    let start_time = std::time::Instant::now();

    let program = intcode::load_program("day11/input.txt").unwrap_or_else(|err| {
        println!("Could not load input file!\n{:?}", err);
        process::exit(1);
    });

    let part_1_robot = run_paint_sequence(&program, false);
    let part_2_robot = run_paint_sequence(&program, true);
    let mut picture = String::new();
    for (y, x) in iproduct!(
        (part_2_robot.min_y..=part_2_robot.max_y),
        (part_2_robot.min_x..=part_2_robot.max_x)
    ) {
        if x == part_2_robot.min_x {
            picture.push('\n');
        }
        match part_2_robot.get_color_at(x, y) {
            1 => picture.push('#'),
            _ => picture.push(' '),
        };
    }

    println!(
        "Part 1: {}\nPart 2: {}\nTime: {}ms",
        part_1_robot.count_painted_panels(),
        picture,
        start_time.elapsed().as_millis()
    );
}

fn run_paint_sequence(program: &[i64], paint_current_panel: bool) -> Robot {
    let (in_send, in_recv) = mpsc::channel();
    let (out_send, out_recv) = mpsc::channel();
    let mut computer = intcode::Computer::new(program, in_recv, out_send);
    std::thread::spawn(move || {
        computer.run().unwrap_or_else(|e| {
            println!("Computer failed: {}", e);
            process::exit(1);
        });
    });

    let mut robot = Robot::new();
    if paint_current_panel {
        robot.paint(1);
    }

    loop {
        if in_send.send(robot.get_current_color()).is_err() {
            break;
        }
        match out_recv.recv() {
            Ok(val) => match val {
                0 | 1 => robot.paint(val),
                _ => {
                    println!("Invalid paint instruction received: {}", val);
                    process::exit(1);
                }
            },
            Err(_) => break,
        };
        match out_recv.recv() {
            Ok(val) => match val {
                0 => robot.turn_and_move(Turns::Left),
                1 => robot.turn_and_move(Turns::Right),
                _ => {
                    println!("Invalid turn instruction received: {}", val);
                    process::exit(1);
                }
            },
            Err(_) => break,
        };
    }

    robot
}

#[derive(Clone, Copy)]
enum Turns {
    Left,
    Right,
}

enum Directions {
    Up,
    Down,
    Left,
    Right,
}

struct Robot {
    x: isize,
    y: isize,
    min_x: isize,
    min_y: isize,
    max_x: isize,
    max_y: isize,
    dir: Directions,
    panels: HashMap<String, Panel>,
}

impl Robot {
    fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            min_x: 0,
            min_y: 0,
            max_x: 0,
            max_y: 0,
            dir: Directions::Up,
            panels: HashMap::new(),
        }
    }

    fn count_painted_panels(&self) -> usize {
        //self.panels.values().map(|panel| match panel.painted { false => 0, true => 1 }).sum()
        self.panels.values().filter(|panel| panel.painted).count()
    }

    fn paint(&mut self, color: i64) {
        self.get_current_panel().paint(color);
    }

    fn get_current_color(&mut self) -> i64 {
        self.get_current_panel().color
    }

    fn turn_and_move(&mut self, turn: Turns) {
        self.dir = match turn {
            Turns::Left => match self.dir {
                Directions::Up => Directions::Left,
                Directions::Left => Directions::Down,
                Directions::Down => Directions::Right,
                Directions::Right => Directions::Up,
            },
            Turns::Right => match self.dir {
                Directions::Up => Directions::Right,
                Directions::Right => Directions::Down,
                Directions::Down => Directions::Left,
                Directions::Left => Directions::Up,
            },
        };

        match self.dir {
            Directions::Up => {
                self.y += 1;
                if self.y > self.max_y {
                    self.max_y = self.y;
                };
            }
            Directions::Down => {
                self.y -= 1;
                if self.y < self.min_y {
                    self.min_y = self.y;
                };
            }
            Directions::Left => {
                self.x -= 1;
                if self.x < self.min_x {
                    self.min_x = self.x;
                };
            }
            Directions::Right => {
                self.x += 1;
                if self.x > self.max_x {
                    self.max_x = self.x;
                };
            }
        }
    }

    fn get_current_panel(&mut self) -> &mut Panel {
        let coordinates = format!("({}, {})", self.x, self.y);
        if !self.panels.contains_key(&coordinates) {
            let new_panel = Panel::new();
            self.panels.insert(coordinates.clone(), new_panel);
        }
        self.panels.get_mut(&coordinates).unwrap()
    }

    fn get_color_at(&self, x: isize, y: isize) -> i64 {
        let coordinates = format!("({}, {})", x, y);
        self.panels.get(&coordinates).map_or(0, |panel| panel.color)
    }
}

struct Panel {
    color: i64,
    painted: bool,
}

impl Panel {
    fn new() -> Self {
        Self {
            color: 0,
            painted: false,
        }
    }

    fn paint(&mut self, color: i64) {
        self.color = color;
        self.painted = true;
    }
}
