//use std::collections::HashSet;

fn main() {
    let mut eris = Eris::top_level(
"..#.#
#####
.#...
...#.
##...");

    eris.print_state();
    for _ in 0..200 {
        eris = eris.tick();
        //eris.print_state();
    }

    println!("{}", eris.count());
}

struct Eris {
    state: u32,
    adjacency: [u8; 25],
    child: Option<Box<Eris>>,
}

impl Eris {
    fn top_level(initial_state: &str) -> Eris {
        let state_joined: String = initial_state.lines().collect();
        let mut state = 0;
        for (i, c) in state_joined.chars().enumerate() {
            if c == '#' { state |= 1 << i }
        }
        println!("{:#027b}", state);
        Eris {
            state,
            adjacency: [0_u8; 25],
            child: None,
        }
    }

    fn child() -> Box<Eris> {
        Box::new(Eris {
            state: 0,
            adjacency: [0_u8; 25],
            child: None,
        })
    }

    fn parent(self) -> Eris {
        Eris {
            state: 0,
            adjacency: [0_u8; 25],
            child: Some(Box::new(self)),
        }
    }

    fn count(&self) -> u32 {
        self.state.count_ones() + self.child.as_ref().map_or(0, |child| child.count())
    }

    fn tick(self) -> Eris {
        let mut top_level = match self.state {
            0 => self,
            _ => Eris::parent(self),
        };
        top_level.calc_new_state();
        top_level
    }

    fn calc_new_state(&mut self) {
        if self.child.is_none() { self.child = Some(Eris::child()); }

        self.calc_adjacency();

        let child = self.child.as_mut().unwrap();
        if self.state > 0 || child.state > 0 { child.calc_new_state(); }
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
                if i > 4 { self.adjacency[i - 5] += 1; }
                if i < 20 { self.adjacency[i + 5] += 1; }
                if i % 5 != 0 { self.adjacency[i - 1] += 1; }
                if i % 5 != 4 { self.adjacency[i + 1] += 1; }
                if (i == 7) || (i == 11) || (i == 13) || (i == 17) { self.increment_child_adjacency(i); }
            }

            if self.child.as_ref().unwrap().state & 1 << i > 0 {
                if i < 5 { self.adjacency[7] += 1; }
                if i >= 20 { self.adjacency[17] += 1; }
                if i % 5 == 0 { self.adjacency[11] += 1; }
                if i % 5 == 4 { self.adjacency[13] += 1; }
            }
        }
        self.adjacency[12] = 0;
    }

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

    fn print_state(&self) {
        let mut display = String::new();
        for i in 0..25 {
            display.push(match (i, self.state & 1 << i) {
                (12, _) => ' ',
                (_, 0) => '.',
                (_, _) => '#',
            });
            if i % 5 == 4 { display.push('\n'); }
        }
        println!("{}", display);

        if let Some(child) = self.child.as_ref() {
            println!();
            child.print_state();
        }
    }
}

