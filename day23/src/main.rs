use std::process;
use std::sync::mpsc::{Sender, Receiver, channel};
use std::thread;

use std::collections::HashMap;

use intcode;

fn main() {
    let memory = intcode::load_program("day23/input.txt").unwrap_or_else(|err| {
        println!("Could not load input file!\n{:?}", err);
        process::exit(1);
    });

    let mut map: HashMap<i64, Computer> = HashMap::new();
    for address in 0..50 {
        let computer = Computer::new(&memory, address);
        map.insert(address, computer);
    }

    let mut idle_count = 0;
    let mut previous_nat_packet: Option<Message> = None;
    let mut nat_packet: Option<Message> = None;

    let message = loop {
        let mut queue = Vec::new();
        for computer in map.values_mut() {
            if let Some(msg) = computer.schedule() {
                queue.push(msg);
            }
        }

        if !queue.is_empty() {
            idle_count = 0;
        } else if idle_count < 50 {
            idle_count += 1;
        } else if nat_packet.is_some() {
            // Idle, send NAT packet
            let this = nat_packet.unwrap();
            println!("Transmitting NAT packet ({}, {})", this.x, this.y);
            if let Some(prev) = previous_nat_packet {
                if prev.y == this.y { break this; }
            }
            map.get_mut(&0).unwrap().send(this.clone());
            previous_nat_packet = Some(this);
            nat_packet = None;
        }

        for msg in queue {
            println!("Message to {}: ({}, {})", msg.destination, msg.x, msg.y);
            if msg.destination == 255 {
                nat_packet = Some(msg);
                continue;
            }
            let target = map.get_mut(&msg.destination).unwrap();
            target.send(msg);
        }
    };

    println!("Repeated message to 255: {}, {}", message.x, message.y);
}

#[derive(Clone)]
struct Message {
    destination: i64,
    x: i64,
    y: i64,
}

struct Computer {
    tx: Sender<i64>,
    rx: Receiver<i64>,
}

impl Computer {
    fn new(memory: &Vec<i64>, address: i64) -> Self {
        //let (network_send, network_recv) = channel();
        let (in_send, in_recv) = channel();
        let (out_send, out_recv) = channel();
        let mut computer = intcode::Computer::new(memory, in_recv, out_send);
        thread::spawn(move || {
            computer.run().unwrap_or_else(|e| {
                println!("Computer failed: {}", e);
                process::exit(1);
            });
        });

        in_send.send(address).unwrap();

        // thread::spawn(move || {
        //     loop {
        //         if let Ok(val) = network_recv.try_recv() {
        //             in_send.send(val).unwrap();
        //         } else {
        //             in_send.send(-1).unwrap();
        //         }
        //     }
        // });
    
        Self {
            tx: in_send, //network_send,
            rx: out_recv,
        }
    }

    fn schedule(&mut self) -> Option<Message> {
        self.tx.send(-1).unwrap();

        match self.rx.try_recv() {
            Ok(-1) => None,
            Ok(dest) => Some(Message{
                destination: dest,
                x: self.rx.recv().unwrap(),
                y: self.rx.recv().unwrap(),
            }),
            Err(_) => None,
        }
    }

    fn send(&mut self, msg: Message) {
        self.tx.send(msg.x).unwrap();
        self.tx.send(msg.y).unwrap();
    }
}