use std::collections::HashMap;
use std::sync::mpsc;
use std::cmp;

use intcode;
#[macro_use]
extern crate itertools;

fn main() {
    let start_time = std::time::Instant::now();

    let program = intcode::load_program("day11/input.txt").unwrap_or_else(|err| {
        println!("Could not load input file!\n{:?}", err);
        std::process::exit(1);
    });

    let part_1_robot = run_paint_sequence(&program, false);
    let part_2_robot = run_paint_sequence(&program, true);
    let picture = part_2_robot.draw_painting();

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
        computer.run();
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
                _ => unreachable!("Invalid paint instruction received: {}", val),
            },
            Err(_) => break,
        };
        match out_recv.recv() {
            Ok(val) => match val {
                0 => robot.turn_and_move(Turn::Left),
                1 => robot.turn_and_move(Turn::Right),
                _ => unreachable!("Invalid turn instruction received: {}", val),
            },
            Err(_) => break,
        };
    }

    robot
}

#[derive(Clone, Copy)]
enum Turn {
    Left,
    Right,
}

#[derive(Clone, Copy)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn turn(self, turn: Turn) -> Self {
        match turn {
            Turn::Left => match self {
                Self::Up => Self::Left,
                Self::Left => Self::Down,
                Self::Down => Self::Right,
                Self::Right => Self::Up,
            },
            Turn::Right => match self {
                Self::Up => Self::Right,
                Self::Right => Self::Down,
                Self::Down => Self::Left,
                Self::Left => Self::Up,
            },
        }
    }
}

struct Robot {
    x: isize,
    y: isize,
    dir: Direction,
    panels: HashMap<(isize, isize), Panel>,
}

impl Robot {
    fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            dir: Direction::Up,
            panels: HashMap::new(),
        }
    }

    fn count_painted_panels(&self) -> usize {
        self.panels.values().filter(|panel| panel.painted).count()
    }

    fn paint(&mut self, color: i64) {
        self.get_current_panel().paint(color);
    }

    fn get_current_color(&mut self) -> i64 {
        self.get_current_panel().color
    }

    fn turn_and_move(&mut self, turn: Turn) {
        self.dir = self.dir.turn(turn);

        match self.dir {
            Direction::Up => {
                self.y -= 1;
            }
            Direction::Down => {
                self.y += 1;
            }
            Direction::Left => {
                self.x -= 1;
            }
            Direction::Right => {
                self.x += 1;
            }
        }
    }

    fn get_current_panel(&mut self) -> &mut Panel {
        self.panels.entry((self.x, self.y)).or_insert(Panel::new())
    }

    fn get_color_at(&self, x: isize, y: isize) -> i64 {
        self.panels.get(&(x, y)).map_or(0, |panel| panel.color)
    }

    fn draw_painting(&self) -> String {
        // Calculate boundaries
        let (min_x, min_y, max_x, max_y) = self.panels.keys().fold((0,0,0,0), |(min_x, min_y, max_x, max_y), &(x, y)| {
            let min_x = cmp::min(min_x, x);
            let min_y = cmp::min(min_y, y);
            let max_x = cmp::max(max_x, x);
            let max_y = cmp::max(max_y, y);
            (min_x, min_y, max_x, max_y)
        });

        let mut picture = String::new();
        for (y, x) in iproduct!((min_y..=max_y), (min_x..=max_x)) {
            if x == min_x {
                picture.push('\n');
            }
            match self.get_color_at(x, y) {
                1 => picture.push('#'),
                _ => picture.push(' '),
            };
        }
        picture
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
