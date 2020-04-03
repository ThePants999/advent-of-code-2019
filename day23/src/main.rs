#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(unused_variables)]
#![allow(dead_code)]

use std::collections::{HashMap, VecDeque};
use std::iter::FromIterator;
use std::time::{Duration, Instant};

extern crate itertools;
use itertools::Itertools;

use tokio::stream::{Stream, StreamExt};
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::{self, Sender};

use intcode::{self, SynchronousComputeResult, SynchronousComputer};

#[tokio::main]
async fn main() {
    let start_time = Instant::now();

    let program = intcode::load_program("day23/input.txt").unwrap_or_else(|err| {
        println!("Could not load input file!\n{:?}", err);
        std::process::exit(1);
    });

    // Today, you get not one, but two implementations.  The default implementation uses a
    // SynchronousComputer for efficiency, and is faster.  But I wanted to try out async
    // code as well, so I also put it in an async wrapper with communication via streams -
    // it's slower, but shows how this might work if that's what you had to do.
    let network = Network::new(50, &program);
    let (part_1_answer, part_2_answer) = network.run();
    //let async_network = AsyncNetwork::new(50, &program);
    //let (part_1_answer, part_2_answer) = async_network.run().awaicat;

    println!(
        "Part 1: {}\nPart 2: {}\nTime: {}ms",
        part_1_answer,
        part_2_answer,
        start_time.elapsed().as_millis()
    );
}

struct Message {
    destination: i64,
    x: i64,
    y: i64,
}

// Simulate the NAT from the question - a device that is sent messages, and then echoes
// the last one sent to it back to computer 0 when the rest of the network is idle.
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
            Some(Message {
                destination: 0,
                x,
                y: self.stored_y,
            })
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

        Self {
            computers,
            nat: NAT::new(),
        }
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
                    self.nat.push(&Message {
                        destination: 255,
                        x,
                        y,
                    });
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

// An adapter on top of an Intcode SynchronousComputer, that allows asynchronous
// communication with the computer via streams.  In order to allow all 50 computers
// to share an output stream, we also need to adapt the interface so that items on
// the queue are `Message`s rather than their constituent parts - we don't want two
// computers interleaving the pieces of a `Message` on the stream.
struct AsyncComputerAdapter {
    to_adapter: Sender<Message>,
}

impl AsyncComputerAdapter {
    fn new(program: &[i64], address: i64, mut to_network: Sender<Message>) -> Self {
        let (to_adapter, mut from_network) = mpsc::channel::<Message>(1);

        // Run the computer as a Tokio task that's constantly trying to progress the
        // computer, yielding when the computer asks for input, and providing it
        // either the next message that's been sent to it, or -1 if nothing has.
        let mut computer = SynchronousComputer::new(program);
        tokio::spawn(async move {
            let mut inputs = Vec::new();
            inputs.push(address);

            loop {
                let output = computer.run(&inputs);
                inputs.clear();
                match output.result {
                    SynchronousComputeResult::ProgramEnded => {
                        panic!("Network computer terminated unexpectedly!")
                    }
                    SynchronousComputeResult::InputRequired => {
                        tokio::task::yield_now().await;
                        match from_network.try_recv() {
                            Ok(value) => {
                                inputs.push(value.x);
                                inputs.push(value.y);
                            }
                            Err(TryRecvError::Empty) => {
                                inputs.push(-1);
                            }
                            Err(TryRecvError::Closed) => {
                                break;
                            }
                        }
                    }
                }

                // Convert triplets of outputs into `Message`s.
                for message in output.outputs.iter().batching(|it| {
                    if let Some(destination) = it.next() {
                        let x = it.next().expect("Computer sent partial message, missing x");
                        let y = it.next().expect("Computer sent partial message, missing y");
                        Some(Message {
                            destination: *destination,
                            x: *x,
                            y: *y,
                        })
                    } else {
                        None
                    }
                }) {
                    let _ = to_network.send(message).await;
                }
            }
        });

        AsyncComputerAdapter { to_adapter }
    }
}

struct AsyncNetwork {
    computers: HashMap<i64, AsyncComputerAdapter>,
    from_computers: Box<dyn Stream<Item = Result<Message, tokio::time::Elapsed>> + Unpin>,
    nat: NAT,
}

impl AsyncNetwork {
    fn new(num_computers: i64, computer_program: &[i64]) -> Self {
        let mut computers = HashMap::with_capacity(num_computers as usize);
        let (send, recv) = mpsc::channel(1);

        for i in 0..num_computers {
            computers.insert(
                i,
                AsyncComputerAdapter::new(computer_program, i, send.clone()),
            );
        }

        // We've just given 50 computers access to the send half of our stream. We can be pretty
        // confident, therefore, that if none of them send anything pretty damn promptly, it's
        // because the network is idle.  Using timeout() turns our output stream into one that
        // will return Err if there's nothing else on the stream.
        let from_computers = Box::new(recv.timeout(Duration::from_micros(1)));

        AsyncNetwork {
            computers,
            from_computers,
            nat: NAT::new(),
        }
    }

    async fn run(mut self) -> (i64, i64) {
        let mut first_y = None;
        let mut last_sent_y = None;

        loop {
            // Pull things off the combined all-computer output stream, and send them as
            // inputs to the right places, until the stream returns Err, which means that
            // the network is idle.
            while let Ok(Some(message)) = self.from_computers.try_next().await {
                if message.destination == 255 {
                    // Send to the NAT.
                    if first_y.is_none() {
                        // Got the answer to part 1.
                        first_y = Some(message.y);
                    }
                    self.nat.push(&message);
                } else {
                    let destination = self.computers.get_mut(&message.destination).unwrap();
                    let _ = destination.to_adapter.send(message).await;
                }
            }

            // Idle - poke the NAT.
            let message = self.nat.pop();
            if let Some(message) = message {
                if let Some(last_y) = last_sent_y {
                    if last_y == message.y {
                        // Found the answer to part 2
                        break;
                    }
                }
                last_sent_y = Some(message.y);

                let destination = self.computers.get_mut(&message.destination).unwrap();
                let _ = destination.to_adapter.send(message).await;
            }
        }

        (first_y.unwrap(), last_sent_y.unwrap())
    }
}
