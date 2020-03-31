use std::collections::HashSet;

const INITIAL_STATE: &str = 
"..#.#
#####
.#...
...#.
##...";

// This is a good'un IMO.  Runs in just 4ms and I'm pretty happy with the code.

fn main() {
    let start_time = std::time::Instant::now();

    // Part 1: simulate a non-recursive Eris, and run it until we see the same state twice.
    // (This program tracks the state in a bitwise fashion, so the biodiversity rating IS
    // the state, we don't need to calculate it separately.)
    let mut single_eris = Eris::new(INITIAL_STATE, false);
    let mut ratings = HashSet::new();
    while ratings.insert(single_eris.state) {
        single_eris = single_eris.tick();
    }
    let part_1_answer = single_eris.state;

    // Part 2: simulate a recursive Eris for 200 ticks.
    let mut multi_eris = Eris::new(INITIAL_STATE, true);
    for _ in 0..200 {
        multi_eris = multi_eris.tick();
    }
    let part_2_answer = multi_eris.count();

    println!(
        "Part 1: {}\nPart 2: {}\nTime: {}ms",
        part_1_answer,
        part_2_answer,
        start_time.elapsed().as_millis()
    );
}

// This represents a single level of Eris (which means the whole thing, for part 1.)
// `state` uses a single bit for each tile, with the least significant bit for the
// top-left tile, and going left-to-right then top-to-bottom.  That happily means
// that the whole variable interpreted as a 25+ bit integer IS the biodiversity rating.
//
// `adjacency` tracks, for each tile, the number of bugs in adjacent tiles. It's reset
// on every tick, but is persistent because we need to calculate that for multiple 
// levels before we decide whether bugs live/die/appear for a given level.
//
// We have a unidirectional link downwards - calculation each tick starts at the top
// and progresses downwards, so we don't need to know about our parent.
struct Eris {
    state: u32,
    adjacency: [u8; 25],
    multi_level: bool,
    child: Option<Box<Eris>>,
}

impl Eris {
    // Create the first Eris level.
    fn new(initial_state: &str, multi_level: bool) -> Eris {
        let state_joined: String = initial_state.lines().collect();
        let mut state = 0;
        for (i, c) in state_joined.chars().enumerate() {
            if c == '#' {
                state |= 1 << i
            }
        }
        Eris {
            state,
            adjacency: [0_u8; 25],
            multi_level,
            child: None,
        }
    }

    // Create a new bottom-level Eris.
    fn child() -> Box<Eris> {
        Box::new(Eris {
            state: 0,
            adjacency: [0_u8; 25],
            multi_level: true,
            child: None,
        })
    }

    // Create a new top-level Eris.
    fn parent(self) -> Eris {
        Eris {
            state: 0,
            adjacency: [0_u8; 25],
            multi_level: true,
            child: Some(Box::new(self)),
        }
    }

    // Determine the number of bugs. (Recurses through all levels.)
    fn count(&self) -> u32 {
        self.state.count_ones() + self.child.as_ref().map_or(0, |child| child.count())
    }

    // Run a tick of the simulation. We need to start at what will be the top level by the
    // end of the tick, so if the current top level has any bugs in it, create a new level
    // above it for safety. This function consumes self and returns an Eris in order to
    // replace the caller's variable with the new parent if we create one, to make sure the
    // caller always owns the top-level Eris.
    fn tick(self) -> Eris {
        let mut top_level = if self.multi_level {
            match self.state {
                0 => self,
                _ => Eris::parent(self),
            }
        } else {
            self
        };
        top_level.calc_new_state();
        top_level
    }

    fn calc_new_state(&mut self) {
        // This function is going to recurse down through children to update everyone's
        // state.  We begin by making sure we have a child, but that doesn't create an
        // infinite loop because we stop when there are no bugs.
        if self.multi_level && self.child.is_none() {
            self.child = Some(Eris::child());
        }

        // Increment adjacency counts on this level.  Doing this first, then calling through
        // to children, THEN updating state, results in adjacency counts being updated on
        // everything all the way down, before new state calculations are done on everything
        // on the way back up the stack.
        self.calc_adjacency();

        if self.multi_level {
            let child = self.child.as_mut().unwrap();
            if self.state > 0 || child.state > 0 {
                child.calc_new_state();
            }
        }

        // We now have a definitive record of how many bugs are adjacent to each tile on this
        // level, so we can apply the rules to determine the new state.
        let mut bug_survival = !0;
        let mut bug_appearance = 0;
        for i in 0..25 {
            bug_survival &= match self.adjacency[i] {
                1 => !0,
                _ => !(1 << i),
            };
            bug_appearance |= match self.adjacency[i] {
                1 | 2 => 1 << i,
                _ => 0,
            }
        }

        self.state = (self.state & bug_survival) | (!self.state & bug_appearance);
        self.adjacency = [0; 25];
    }

    // Calculates our own adjacency counts from our own state and our child's
    // state, and records our own impact on our child.  Assumes our parent's
    // impact on us has already been calculated, and our child's impact on itself
    // and its child is left for later recursion.
    fn calc_adjacency(&mut self) {
        for i in 0..25 {
            if self.state & 1 << i > 0 {
                if i > 4 {
                    self.adjacency[i - 5] += 1;
                }
                if i < 20 {
                    self.adjacency[i + 5] += 1;
                }
                if i % 5 != 0 {
                    self.adjacency[i - 1] += 1;
                }
                if i % 5 != 4 {
                    self.adjacency[i + 1] += 1;
                }
                if self.multi_level && ((i == 7) || (i == 11) || (i == 13) || (i == 17)) {
                    self.increment_child_adjacency(i);
                }
            }

            if let Some(child) = self.child.as_ref() {
                if child.state & 1 << i > 0 {
                    if i < 5 {
                        self.adjacency[7] += 1;
                    }
                    if i >= 20 {
                        self.adjacency[17] += 1;
                    }
                    if i % 5 == 0 {
                        self.adjacency[11] += 1;
                    }
                    if i % 5 == 4 {
                        self.adjacency[13] += 1;
                    }
                }
                self.adjacency[12] = 0;
            }
        }
    }

    // Having detected a bug at `location`, which is adjacent to the central "tile",
    // increment the appropriate adjacency counts on our child.
    fn increment_child_adjacency(&mut self, location: usize) {
        for i in match location {
            7 => (0..=4).step_by(1),
            11 => (0..=20).step_by(5),
            13 => (4..=24).step_by(5),
            17 => (20..=24).step_by(1),
            _ => panic!("Attempt to increment child adjacency from invalid location"),
        } {
            self.child.as_mut().unwrap().adjacency[i] += 1;
        }
    }

    fn _print_state(&self) {
        let mut display = String::new();
        for i in 0..25 {
            display.push(match (i, self.state & 1 << i) {
                (12, _) => ' ',
                (_, 0) => '.',
                (_, _) => '#',
            });
            if i % 5 == 4 {
                display.push('\n');
            }
        }
        println!("{}", display);

        if let Some(child) = self.child.as_ref() {
            println!();
            child._print_state();
        }
    }
}
