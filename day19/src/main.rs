#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]

use intcode;
#[macro_use]
extern crate lazy_static;

struct Row {
    first_col: Option<i64>,
    last_col: Option<i64>,
}

impl Row {
    // For part 1, we need to ignore anything beyond the first 50 columns.
    fn count_part_1_affected_points(&self) -> i64 {
        if let Some(first) = self.first_col {
            std::cmp::max(std::cmp::min(self.last_col.unwrap(), 49) - first + 1, 0)
        } else {
            0
        }
    }

    fn count_affected_points(&self) -> i64 {
        if let Some(first) = self.first_col {
            self.last_col.unwrap() - first + 1
        } else {
            0
        }
    }
}

fn main() {
    let start_time = std::time::Instant::now();

    let mut rows: Vec<Row> = Vec::new();
    let mut y = 0;
    let mut first_possible_finish = 0;
    let mut part_1_answer = 0;
    let part_2_answer = loop {
        let row = scan_row(y, rows.get((y - 1) as usize));

        // A 90-degree angle on the tractor beam is pretty unlikely, but for safety,
        // let's assume that the first row we see with 100 affected points is a
        // candidate for the first 100x100 area.  (We'll actually start checking for
        // a valid 100x100 area at the _bottom_ row of the area.)
        if first_possible_finish == 0 && row.count_affected_points() >= 100 {
            first_possible_finish = y + 99;
        }

        if first_possible_finish > 0 && y >= first_possible_finish {
            // If a 100x100 area that horizontally starts at the beginning of this row
            // also fits in 99 rows up, it's the answer to part 2.
            if row.first_col.unwrap() + 99 == rows[(y - 99) as usize].last_col.unwrap() {
                break (row.first_col.unwrap() * 10000) + (y - 99);
            }
        }

        rows.push(row);
        y += 1;

        // Once we've finished the first 50 rows, get the part 1 answer.
        if y == 50 {
            part_1_answer = rows
                .iter()
                .map(Row::count_part_1_affected_points)
                .sum::<i64>();
        }
    };

    println!(
        "Part 1: {}\nPart 2: {}\nTime: {}ms",
        part_1_answer,
        part_2_answer,
        start_time.elapsed().as_millis()
    );
}

// Deploy a drone to the specified co-ordinates to see whether the tractor beam affects it.
fn point_affected(x: i64, y: i64) -> bool {
    lazy_static! {
        static ref PROGRAM: Vec<i64> =
            intcode::load_program("day19/input.txt").unwrap_or_else(|err| {
                println!("Could not load input file!\n{:?}", err);
                std::process::exit(1);
            });
    }

    let outputs = intcode::run_computer(&PROGRAM, &[x, y]);
    outputs[0] == 1
}

// Find the first and last affected columns in a row, by searching from the same columns as the
// previous row's first and last.
fn scan_row(y: i64, previous_row: Option<&Row>) -> Row {
    let (start_col, previous_end) = if let Some(previous_row) = previous_row {
        if let Some(previous_start) = previous_row.first_col {
            (previous_start, previous_row.last_col.unwrap())
        } else {
            (0, 0)
        }
    } else {
        (0, 0)
    };

    let mut x = start_col;
    let first_col = loop {
        if point_affected(x, y) {
            break Some(x);
        }

        x += 1;
        if x == start_col + 8 {
            // This row looks empty, let's give up on it
            break None;
        }
    };

    let last_col = if let Some(first_col) = first_col {
        x = std::cmp::max(first_col, previous_end);
        loop {
            if !point_affected(x, y) {
                break Some(x - 1);
            }
            x += 1;
        }
    } else {
        None
    };

    Row {
        first_col,
        last_col,
    }
}
