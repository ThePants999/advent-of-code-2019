#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]

use std::collections::{HashMap, VecDeque};
use std::iter::FromIterator;

use intcode;

fn main() {
    let start_time = std::time::Instant::now();

    let program = intcode::load_program("day23/input.txt").unwrap_or_else(|err| {
        println!("Could not load input file!\n{:?}", err);
        std::process::exit(1);
    });

    let network = Network::new(50, &program);
    let (part_1_answer, part_2_answer) = network.run();

    println!("Part 1: {}\nPart 2: {}\nTime: {}ms", part_1_answer, part_2_answer, start_time.elapsed().as_millis());
}

struct Message {
    destination: i64,
    x: i64,
    y: i64,
}

#[derive(Default)]
struct NAT {
    stored_x: Option<i64>,
    stored_y: i64,
}

impl NAT {
    fn new() -> Self {
        Self::default()
    }

    fn push(&mut self, message: &Message) {
        assert!(message.destination == 255);
        self.stored_x = Some(message.x);
        self.stored_y = message.y;
    }

    fn pop(&self) -> Option<Message> {
        if let Some(x) = self.stored_x {
            Some(Message { destination: 0, x, y: self.stored_y })
        } else {
            None
        }
    }
}

struct ComputerLink {
    computer: intcode::SynchronousComputer,
    outbound_messages: Vec<i64>,
}

impl ComputerLink {
    // Create an Intcode computer and initialise its input queue with its address.
    fn new(program: &[i64], address: i64) -> Self {
        Self {
            computer: intcode::SynchronousComputer::new(program),
            outbound_messages: vec![address],
        }
    }
}

struct Network {
    computers: HashMap<i64, ComputerLink>,
    nat: NAT,
}

impl Network {
    fn new(num_computers: i64, computer_program: &[i64]) -> Self {
        let mut computers = HashMap::with_capacity(num_computers as usize);
        for i in 0..num_computers {
            computers.insert(i, ComputerLink::new(computer_program, i));
        }

        Self { computers, nat: NAT::new() }
    }

    fn run(mut self) -> (i64, i64) {
        let mut first_y = None;
        let mut last_sent_y = None;

        loop {
            let mut outputs = Vec::new();

            // Transmit to and receive from every computer.  We'll transmit everything that's
            // queued, and if that's "nothing", transmit -1.
            for link in self.computers.values_mut() {
                if link.outbound_messages.is_empty() {
                    link.outbound_messages.push(-1);
                }
                let inputs: Vec<i64> = link.outbound_messages.drain(..).collect();
                let mut compute_output = link.computer.run(&inputs);
                assert!(compute_output.result == intcode::SynchronousComputeResult::InputRequired);

                // Gather all of the outputs together.
                outputs.append(&mut compute_output.outputs);
            }

            if outputs.is_empty() {
                // No computer had any output for us - poke the NAT.
                let message = self.nat.pop();
                if let Some(message) = message {
                    if let Some(last_y) = last_sent_y {
                        if last_y == message.y {
                            // Found the answer to part 2
                            break;
                        }
                    }
                    last_sent_y = Some(message.y);

                    outputs.push(message.destination);
                    outputs.push(message.x);
                    outputs.push(message.y);    
                }
            }

            let mut queue = VecDeque::from_iter(outputs);
            while !queue.is_empty() {
                let destination = queue.pop_front().unwrap();
                let x = queue.pop_front().unwrap();
                let y = queue.pop_front().unwrap();
                if destination == 255 {
                    // Send to the NAT.
                    if first_y.is_none() {
                        // Got the answer to part 1.
                        first_y = Some(y);
                    }
                    self.nat.push(&Message { destination: 255, x, y });                    
                } else {
                    // Send to another computer.
                    let link = self.computers.get_mut(&destination).unwrap();
                    link.outbound_messages.push(x);
                    link.outbound_messages.push(y);
                }
            }
        }

        (first_y.unwrap(), last_sent_y.unwrap())
    }
}