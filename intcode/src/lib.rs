//! # Intcode
//!
//! `intcode` is a library for executing Intcode programs, as featured in the Advent
//! of Code 2019. To use this library, create a [`Computer`], which takes a program plus
//! two channels, one for sending inputs in and one for receiving outputs.  Helper
//! functions assist with loading programs from file, and executing them simplistically.
//!
//! Typical usage might look like this:
//!
//! ```
//! let program = intcode::load_program("path/to/input.txt").unwrap_or_else(|err| {
//!     println!("Could not load input file!\n{:?}", err);
//!     process::exit(1);
//! });
//!
//! let (in_send, in_recv) = std::sync::mpsc::channel();
//! let (out_send, out_recv) = std::sync::mpsc::channel();
//! let mut computer = intcode::Computer::new(&program, in_recv, out_send);
//! std::thread::spawn(move || {
//!     computer.run().unwrap_or_else(|e| {
//!         println!("Computer failed: {}", e);
//!         process::exit(1);
//!     });
//! });
//! ```
//!
//! The computer is now executing in parallel, and you can communicate with it
//! via `in_send` and `out_recv`.
//!
//! [`Computer`]: ./struct.Computer.html
//!

#![crate_name = "intcode"]
#![crate_type = "lib"]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]

use std::fs::File;
use std::io;
use std::io::Read;
use std::sync::mpsc::{self, Receiver, Sender};

// Operations that the Intcode computer can perform.
#[derive(PartialEq, Eq)]
enum OperationType {
    Add,
    Multiply,
    Input,
    Output,
    JumpIfTrue,
    JumpIfFalse,
    LessThan,
    Equals,
    RelativeBaseOffset,
    End,
}

impl OperationType {
    // Decode an opcode into an enum value.
    fn from_opcode(opcode: i64) -> Self {
        match opcode {
            1 => OperationType::Add,
            2 => OperationType::Multiply,
            3 => OperationType::Input,
            4 => OperationType::Output,
            5 => OperationType::JumpIfTrue,
            6 => OperationType::JumpIfFalse,
            7 => OperationType::LessThan,
            8 => OperationType::Equals,
            9 => OperationType::RelativeBaseOffset,
            99 => OperationType::End,
            bad_code => unreachable!("Invalid opcode detected: {}", bad_code),
        }
    }

    // Determine the complete size of an instruction of this type - i.e. the distance the
    // instruction pointer needs to move to get to the next instruction.
    fn instruction_size(&self) -> i64 {
        match self {
            OperationType::Add
            | OperationType::Multiply
            | OperationType::LessThan
            | OperationType::Equals => 4,
            OperationType::Input | OperationType::Output | OperationType::RelativeBaseOffset => 2,
            OperationType::JumpIfTrue | OperationType::JumpIfFalse => 3,
            OperationType::End => 0,
        }
    }
}

// Each parameter in an operation can work in one of three modes.
//
// -  An "immediate" parameter means that the value of the parameter
//    is the number that should be used in the operation.
// -  A "position" parameter means that the value of the parameter is
//    a memory address, and the content of that address is the number
//    that should be used in the operation.
// -  A "relative" parameter means that the value of the parameter is
//    a delta to the current relative base, and combining the two gives
//    a memory address whose content is the number that should be used
//    in the operation.
enum ParameterMode {
    Position,
    Immediate,
    Relative,
}

impl ParameterMode {
    // The instruction (i.e. the value at the instruction pointer) contains more than
    // just the opcode.  It also specifies which mode the following parameters should
    // work in. Each parameter's mode is encoded in a different base 10 digit of the
    // instruction - the lowest two digits are the opcode, and the next three are the
    // parameter modes for the three parameters.
    fn from_instruction(instruction: i64, parameter_num: i64) -> Self {
        let divisor = match parameter_num {
            1 => 100,
            2 => 1_000,
            3 => 10_000,
            _ => unreachable!(),
        };
        match (instruction / divisor) % 10 {
            0 => ParameterMode::Position,
            1 => ParameterMode::Immediate,
            2 => ParameterMode::Relative,
            _ => unreachable!("Bad instruction found in program: {}", instruction),
        }
    }
}

struct Parameters {
    a: i64,
    b: i64,
    c: i64,
}

struct Operation {
    optype: OperationType,
    params: Parameters,
}

/// A virtual computer whose memory contains an Intcode program and which can execute
/// said program.
pub struct Computer {
    memory: Vec<i64>,
    in_channel: Receiver<i64>,
    out_channel: Sender<i64>,
    instruction_pointer: i64,
    relative_base: i64,
}

impl Computer {
    /// Construct a computer to run `program`.  `program` only needs to be as long as the
    /// instructions and data contained within it; the computer has additional memory available
    /// that the program can refer to.
    ///
    /// `in_channel` is the receive half of a channel on which you can send runtime inputs to
    /// the program, and `out_channel` is the send half of a channel on which you can receive
    /// runtime outputs.
    pub fn new(program: &[i64], in_channel: Receiver<i64>, out_channel: Sender<i64>) -> Self {
        Self {
            memory: program.into(),
            in_channel,
            out_channel,
            instruction_pointer: 0,
            relative_base: 0,
        }
    }

    /// Executes the program in the computer's memory.
    ///
    /// This will synchronously run the program through to completion on the current thread, but
    /// may block waiting for input on the computer's input channel if sufficient inputs are not
    /// pre-sent.
    ///
    /// # Panics
    ///
    /// Panics if any problem is hit executing the program, which would indicate either that the
    /// program is invalid, or that invalid inputs were provided to it, or that the channels were
    /// closed prematurely.
    pub fn run(&mut self) {
        loop {
            let operation = self.fetch_operation();
            if operation.optype == OperationType::End {
                break;
            }
            self.execute_operation(operation);
        }
    }

    // Execute a single operation that's been fully parsed from memory.
    fn execute_operation(&mut self, op: Operation) {
        match op.optype {
            OperationType::Add => self.set_at_address(op.params.c, op.params.a + op.params.b),
            OperationType::Multiply => self.set_at_address(op.params.c, op.params.a * op.params.b),
            OperationType::Input => self.set_at_address(
                op.params.a,
                self.in_channel
                    .recv()
                    .expect("Intcode computer expected an input but channel was closed!"),
            ),
            OperationType::Output => self
                .out_channel
                .send(op.params.a)
                .expect("Intcode computer tried to send an output but channel was closed!"),
            OperationType::JumpIfTrue => {
                if op.params.a != 0 {
                    self.instruction_pointer = op.params.b;
                }
            }
            OperationType::JumpIfFalse => {
                if op.params.a == 0 {
                    self.instruction_pointer = op.params.b;
                }
            }
            OperationType::LessThan => self.set_at_address(op.params.c, op.params.a < op.params.b),
            OperationType::Equals => self.set_at_address(op.params.c, op.params.a == op.params.b),
            OperationType::RelativeBaseOffset => self.relative_base += op.params.a,
            OperationType::End => unreachable!(),
        }
    }

    // Determine the mode in which to evaluate a given parameter.
    fn get_parameter_mode(&self, parameter_num: i64) -> ParameterMode {
        ParameterMode::from_instruction(
            self.fetch_from_address(self.instruction_pointer),
            parameter_num,
        )
    }

    // Load a parameter that we're going to use as a piece of data.  This implies
    // the "standard" treatment for all parameter types, meaning that for non-"Immediate"
    // parameters, we treat the value as an address, and then go and pick up the data
    // from that address.
    fn fetch_read_parameter(&self, parameter_num: i64) -> i64 {
        let value = self.fetch_from_address(self.instruction_pointer + parameter_num);
        match self.get_parameter_mode(parameter_num) {
            ParameterMode::Position => self.fetch_from_address(value),
            ParameterMode::Immediate => value,
            ParameterMode::Relative => self.fetch_from_address(value + self.relative_base),
        }
    }

    // Load a parameter that we're going to use as a location to write data to.  Our
    // handling of these parameters is slightly different, because we want to return the
    // address, not the data at that address, so there's one fewer level of indirection
    // (which results in "Position" and "Immediate" being treated identically).
    fn fetch_write_parameter(&self, parameter_num: i64) -> i64 {
        let value = self.fetch_from_address(self.instruction_pointer + parameter_num);
        match self.get_parameter_mode(parameter_num) {
            ParameterMode::Position | ParameterMode::Immediate => value,
            ParameterMode::Relative => value + self.relative_base,
        }
    }

    // Load the next operation to perform - that is, the operation type and parameters - from memory.
    fn fetch_operation(&mut self) -> Operation {
        let optype =
            OperationType::from_opcode(self.fetch_from_address(self.instruction_pointer) % 100);

        // For most operations, the first two parameters are data, and the third - if they have a
        // third - is a location to put the result.  We handle those a bit differently.  However,
        // Input is a special case.  It only has one parameter, and it's a location for the result.
        let (a, b, c) = if optype == OperationType::Input {
            (self.fetch_write_parameter(1), 0, 0)
        } else {
            // For simplicity, we'll fetch the maximum three parameters following the instruction.
            // Some operation types don't have three parameters, in which case we'll have parsed 
            // the following instruction as a parameter to this one, but `execute_operation` will 
            // ignore "parameters" it doesn't need so that's not an issue.
            (
                self.fetch_read_parameter(1),
                self.fetch_read_parameter(2),
                self.fetch_write_parameter(3),
            )
        };
        self.instruction_pointer += optype.instruction_size();
        Operation {
            optype,
            params: Parameters { a, b, c },
        }
    }

    /// Safely retrieves the data at a given memory address.
    pub fn fetch_from_address(&self, address: i64) -> i64 {
        *self.memory.get(address as usize).unwrap_or(&0)
    }

    // Stores a value at a memory location, enlarging the memory if needed.
    fn set_at_address<T: Into<i64>>(&mut self, address: i64, value: T) {
        let address = address as usize;
        if address >= self.memory.len() {
            self.memory.resize(address + 1, 0);
        }
        self.memory[address] = value.into();
    }
}

/// Helper function for running an Intcode program (from file) that can run all the way to
/// completion with a predetermined set of inputs (including no inputs).
///
/// # Errors
///
/// Returns an error if the file cannot be opened or read, or if it is syntactically
/// invalid.
///
/// # Panics
///
/// Panics if the program is syntactically valid but semantically invalid.
pub fn load_and_run_computer(path: &str, inputs: &[i64]) -> Result<Vec<i64>, String> {
    let program = load_program(path).map_err(|e| format!("{:?}", e))?;
    Ok(run_computer(&program, inputs))
}

/// Helper function for running an Intcode program that can run all the way to
/// completion with a predetermined set of inputs (including no inputs).
///
/// # Panics
///
/// Panics if the supplied program is invalid.
pub fn run_computer(program: &[i64], inputs: &[i64]) -> Vec<i64> {
    let (in_sender, in_receiver) = mpsc::channel();
    let (out_sender, out_receiver) = mpsc::channel();

    for input in inputs {
        in_sender.send(*input).unwrap();
    }

    Computer::new(&program, in_receiver, out_sender).run();

    let mut outputs = Vec::new();
    while let Ok(output) = out_receiver.try_recv() {
        outputs.push(output);
    }
    outputs
}

/// Loads an Intcode program from the file at `path`.
///
/// # Errors
///
/// Returns an error if the file cannot be opened or read, or if it is syntactically
/// invalid, but doesn't detect if the program itself is invalid.
pub fn load_program(path: &str) -> Result<Vec<i64>, io::Error> {
    let mut input_file = File::open(path)?;
    let mut input = String::new();
    input_file.read_to_string(&mut input)?;
    input
        .split(',')
        .map(|number| {
            number
                .parse::<i64>()
                .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))
        })
        .collect()
}
