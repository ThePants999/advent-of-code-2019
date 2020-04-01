#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]

use intcode;

fn main() {
    let start_time = std::time::Instant::now();

    let mut program = intcode::load_program("day17/input.txt").unwrap_or_else(|err| {
        println!("Could not load input file!\n{:?}", err);
        std::process::exit(1);
    });

    // Run the Intcode computer in camera mode to build a picture of the scaffold.
    let picture = intcode::run_parallel_computer(&program, &[]);

    // Parse the picture to generate a list of the intersections (needed for part 1) and a set
    // of movement instructions to follow it (needed for part 2).
    let scaffold = Scaffold::generate(&picture);

    // We've found each intersection twice, so we'll need to halve the result.
    let part_1_answer = scaffold
        .intersections
        .iter()
        .map(|(row, col)| row * col)
        .sum::<usize>()
        / 2;

    // Split the movement instructions into subroutines and a main routine, and feed them into
    // the Intcode computer in movement mode.
    program[0] = 2;
    let logic = MovementLogic::parse(&scaffold.program);
    let part_2_answer = move_robot(&program, logic);

    println!(
        "Part 1: {}\nPart 2: {}\nTime: {}ms",
        part_1_answer,
        part_2_answer,
        start_time.elapsed().as_millis()
    );
}

// Feed the movement logic into the Intcode computer, and fetch the final output, which is
// the quantity of dust collected.
fn move_robot(program: &[i64], logic: MovementLogic) -> i64 {
    let inputs = logic.into_intcode_inputs();
    let outputs = intcode::run_parallel_computer(program, &inputs);
    let final_output = outputs.last().unwrap();
    assert!(*final_output > 255);
    *final_output
}

struct MovementLogic {
    main_routine: String,
    subroutines: Vec<String>,
}

impl MovementLogic {
    fn parse(full_program: &str) -> Self {
        let mut main_routine = full_program.to_string();
        let mut subroutines = Vec::new();

        loop {
            // Find the longest set of instructions that's repeated later.
            for i in (1..=20).rev() {
                // Skip over any subroutine calls at the beginning.
                let start = std::cmp::min(
                    main_routine.find('L').unwrap(),
                    main_routine.find('R').unwrap(),
                );
                let candidate = &main_routine[start..start + i];

                // Skip anything that contains a subroutine call.
                if candidate.contains('A') || candidate.contains('B') {
                    continue;
                }

                if main_routine[start + i..].contains(candidate) {
                    let mut subroutine = candidate.to_string();
                    let subroutine_name = match subroutines.len() {
                        0 => "A,",
                        1 => "B,",
                        2 => "C,",
                        _ => unreachable!(),
                    };
                    main_routine = main_routine.replace(&subroutine, subroutine_name);
                    subroutine.pop(); // Remove trailing comma
                    subroutines.push(subroutine);
                    break;
                }
            }

            // If there are no movement instructions left, only subroutine calls, we're done.
            if !main_routine.contains('L') && !main_routine.contains('R') {
                break;
            }
        }
        main_routine.pop(); // Remove trailing comma

        Self {
            main_routine,
            subroutines,
        }
    }

    fn into_intcode_inputs(self) -> Vec<i64> {
        let mut inputs = Vec::new();
        self.main_routine
            .chars()
            .for_each(|c| inputs.push(c as i64));
        inputs.push('\n' as i64);
        for subroutine in self.subroutines {
            subroutine.chars().for_each(|c| inputs.push(c as i64));
            inputs.push('\n' as i64);
        }
        inputs.push('n' as i64);
        inputs.push('\n' as i64);
        inputs
    }
}

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn parse(c: char) -> Self {
        match c {
            '^' => Self::Up,
            'v' => Self::Down,
            '<' => Self::Left,
            '>' => Self::Right,
            _ => unreachable!(),
        }
    }

    fn move_one(
        &self,
        row: usize,
        col: usize,
        width: usize,
        height: usize,
    ) -> Result<(usize, usize), ()> {
        match self {
            Self::Up if row > 0 => Ok((row - 1, col)),
            Self::Down if row < (height - 1) => Ok((row + 1, col)),
            Self::Left if col > 0 => Ok((row, col - 1)),
            Self::Right if col < (width - 1) => Ok((row, col + 1)),
            _ => Err(()),
        }
    }

    fn turn_left(&self) -> Self {
        match self {
            Self::Up => Self::Left,
            Self::Left => Self::Down,
            Self::Down => Self::Right,
            Self::Right => Self::Up,
        }
    }

    fn turn_right(&self) -> Self {
        match self {
            Self::Up => Self::Right,
            Self::Right => Self::Down,
            Self::Down => Self::Left,
            Self::Left => Self::Up,
        }
    }
}

struct Scaffold {
    program: String,
    intersections: Vec<(usize, usize)>,
}

#[allow(clippy::filter_map)]
impl Scaffold {
    fn generate(picture: &[i64]) -> Self {
        let start_position = picture
            .iter()
            .position(|&num| num == 60 || num == 62 || num == 94 || num == 118) // <, >, ^, v
            .unwrap();
        let grid: Vec<Vec<char>> = picture
            .split(|num| *num == 10) // Split on newlines
            .filter(|slice| !slice.is_empty()) // Drop blank rows
            .map(|slice| slice.iter().map(|num| *num as u8 as char).collect()) // Convert to chars
            .collect();
        let width = grid[0].len();
        let height = grid.len();
        let mut row = start_position / (width + 1); // + 1 because we calculated start_position
        let mut col = start_position % (width + 1); // before removing newlines
        let mut dir = Direction::parse(grid[row][col]);
        let mut program = String::new();
        let mut intersections = Vec::new();

        loop {
            let mut distance = 0;

            // Go as far as we can along the scaffold in a straight line.
            while let Ok((new_row, new_col)) = dir.move_one(row, col, width, height) {
                if grid[new_row][new_col] == '#' {
                    // We're about to move forwards. Before we do, check to see if the space
                    // we're vacating was an intersection.
                    if distance > 0 {
                        if let Ok((side_row, side_col)) =
                            dir.turn_left().move_one(row, col, width, height)
                        {
                            if grid[side_row][side_col] == '#' {
                                intersections.push((row, col));
                            }
                        }
                    }

                    // We're OK to move forwards.
                    row = new_row;
                    col = new_col;
                    distance += 1;
                } else {
                    // Hit a corner
                    break;
                }
            }

            // Record how far we moved.
            if distance > 0 {
                program += &distance.to_string();
                program.push(',');
            }

            // Figure out whether to turn left or right to continue following the scaffold, and
            // record that too.
            let left_result = dir.turn_left().move_one(row, col, width, height);
            let right_result = dir.turn_right().move_one(row, col, width, height);
            if left_result.is_ok() && grid[left_result.unwrap().0][left_result.unwrap().1] == '#' {
                dir = dir.turn_left();
                program.push('L');
                program.push(',');
            } else if right_result.is_ok()
                && grid[right_result.unwrap().0][right_result.unwrap().1] == '#'
            {
                dir = dir.turn_right();
                program.push('R');
                program.push(',');
            } else {
                // Neither turning left nor right finds continued scaffold - we must have reached the end.
                break;
            }
        }

        Self {
            program,
            intersections,
        }
    }
}
