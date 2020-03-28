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

use std::fs::File;
use std::io;
use std::io::Read;
use std::sync::mpsc::{self, Receiver, Sender};

const MEMORY_CAPACITY: usize = 65536;

#[derive(PartialEq, Eq)]
enum OperationTypes {
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

struct Instruction {
    optype: OperationTypes,
    param_modes: Vec<ParameterModes>,
}

impl Instruction {
    fn decode(mut instruction: i64) -> Result<Self, String> {
        // An instruction comprises multiple pieces of information.
        // 
        // The ones and tens digits of the decimal representation contain the opcode,
        // specifying the type of operation to perform.  Each operation type then
        // requires a fixed number of parameters, and while the parameters themselves
        // are in subsequent memory addresses, the modes in which they operate are
        // encoded as the higher digits of the original instruction - i.e. the hundreds
        // digit specifies the mode of the first parameter, the thousands digit the
        // second, etc.
        let (optype, num_parameters) = match instruction % 100 {
            1 => (OperationTypes::Add, 3),
            2 => (OperationTypes::Multiply, 3),
            3 => (OperationTypes::Input, 1),
            4 => (OperationTypes::Output, 1),
            5 => (OperationTypes::JumpIfTrue, 2),
            6 => (OperationTypes::JumpIfFalse, 2),
            7 => (OperationTypes::LessThan, 3),
            8 => (OperationTypes::Equals, 3),
            9 => (OperationTypes::RelativeBaseOffset, 1),
            99 => (OperationTypes::End, 0),
            opcode => return Err(format!("Invalid opcode in program: {}", opcode)),
        };
        instruction /= 100;

        let mut param_modes = Vec::with_capacity(num_parameters);
        for _ in 0..num_parameters {
            param_modes.push(match instruction % 10 {
                0 => ParameterModes::Position,
                1 => ParameterModes::Immediate,
                2 => ParameterModes::Relative,
                _ => return Err(format!("Invalid parameter type: {}", instruction)),
            });
            instruction /= 10;
        }

        Ok(Instruction{ optype, param_modes })
    }
}

struct Operation {
    instruction: Instruction,
    params: Vec<Parameter>,
}

enum ParameterModes {
    Position,
    Immediate,
    Relative,
}

struct Parameter {
    unresolved_value: i64,
    resolved_value: i64,
}

/// A virtual computer whose memory contains an Intcode program and which can execute
/// said program.
pub struct Computer {
    memory: Box<[i64; MEMORY_CAPACITY]>,
    in_channel: Receiver<i64>,
    out_channel: Sender<i64>,
    instruction_pointer: usize,
    relative_base: usize,
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
        let mut comp = Self {
            memory: Box::new([0; MEMORY_CAPACITY]),
            in_channel,
            out_channel,
            instruction_pointer: 0,
            relative_base: 0,
        };

        for (position, element) in program.iter().enumerate() {
            comp.memory[position] = *element;
        }

        comp
    }

    /// Executes the program in the computer's memory.
    /// 
    /// This will synchronously run the program through to completion on the current thread, but
    /// may block waiting for input on the computer's input channel if sufficient inputs are not
    /// pre-sent.
    /// 
    /// Returns Ok(()) if the program completes successfully, or Err if anything goes wrong.
    pub fn run(&mut self) -> Result<(), String> {
        loop {
            // An instruction consists of an operation code and a set of parameters.
            // The number of parameters is determined by the operation code, and
            // follow the operation code at consecutive memory locations.
            let instruction_val = self.fetch_from_address(self.instruction_pointer)?;
            let instruction = Instruction::decode(instruction_val)?;
            self.instruction_pointer += 1;

            if instruction.optype == OperationTypes::End {
                break Ok(())
            }

            let operation = Operation {
                params: self.extract_parameters(&instruction.param_modes)?,
                instruction,
            };

            self.apply_operation(operation)?;
        }
    }

    fn extract_parameters(&mut self, param_modes: &[ParameterModes]) -> Result<Vec<Parameter>, String> {
        // Each parameter can work in one of three modes.
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
        //
        // The "unresolved" value is the value before a memory lookup, and the
        // resolved value is the value to use in the operation.
        //
        // Note that some operations treat a parameter as a memory address.  In
        // those cases, the unresolved value is always used - we don't support
        // multiple levels of indirection.
        let mut parameters = Vec::new();
        for param_mode in param_modes {
            let mut unresolved_value = self.fetch_from_address(self.instruction_pointer)?;
            let resolved_value = match param_mode {
                ParameterModes::Position => self.fetch_from_address(unresolved_value as usize)?,
                ParameterModes::Immediate => unresolved_value,
                ParameterModes::Relative => {
                    unresolved_value += self.relative_base as i64;
                    self.fetch_from_address(unresolved_value as usize)?
                },
            };
            parameters.push(Parameter { unresolved_value,  resolved_value });
            self.instruction_pointer += 1;
        }
        Ok(parameters)
    }

    fn apply_operation(&mut self, op: Operation) -> Result<(), String> {
        match op.instruction.optype {
            OperationTypes::Add => {
                // Add parameters 0 and 1, and store the result at the address specified
                // by parameter 2.
                let result = op.params[0].resolved_value + op.params[1].resolved_value;
                self.check_address(op.params[2].unresolved_value as usize)?;
                self.memory[op.params[2].unresolved_value as usize] = result;
            },
            OperationTypes::Multiply => {
                // Multiply parameters 0 and 1, and store the result at the address specified
                // by parameter 2.
                let result = op.params[0].resolved_value * op.params[1].resolved_value;
                self.check_address(op.params[2].unresolved_value as usize)?;
                self.memory[op.params[2].unresolved_value as usize] = result;
            },
            OperationTypes::Input => match self.in_channel.recv() {
                // Pull a value from the input stream and store it at the address specified
                // by parameter 0. (Parameter 0 is always treated as immediate.)
                Ok(input) => {
                    self.check_address(op.params[0].unresolved_value as usize)?;
                    self.memory[op.params[0].unresolved_value as usize] = input;
                }
                Err(e) => {
                    return Err(format!("Ran out of inputs!\n{}", e));
                }
            },
            OperationTypes::Output => {
                // Push parameter 0 to the output stream.
                self.out_channel
                    .send(op.params[0].resolved_value)
                    .map_err(|e| format!("{:?}", e))?;
            },
            OperationTypes::JumpIfTrue => {
                // If parameter 0 is non-zero, jump the instruction pointer to parameter 1.
                if op.params[0].resolved_value != 0 {
                    self.check_address(op.params[1].resolved_value as usize)?;
                    self.instruction_pointer = op.params[1].resolved_value as usize;
                }
            },
            OperationTypes::JumpIfFalse => {
                // If parameter 0 is zero, jump the instruction pointer to parameter 1.
                if op.params[0].resolved_value == 0 {
                    self.check_address(op.params[1].resolved_value as usize)?;
                    self.instruction_pointer = op.params[1].resolved_value as usize;
                }
            },
            OperationTypes::LessThan => {
                // Determine whether parameter 0 is less than parameter 1, and store 1 or 0
                // accordingly at the address specified by parameter 2. (Parameter 2 is always
                // treated as immediate.)
                let result = if op.params[0].resolved_value < op.params[1].resolved_value { 1 } else { 0 };
                self.check_address(op.params[2].unresolved_value as usize)?;
                self.memory[op.params[2].unresolved_value as usize] = result;
            },
            OperationTypes::Equals => {
                // Determine whether parameter 0 is equal to parameter 1, and store 1 or 0
                // accordingly at the address specified by parameter 2. (Parameter 2 is always
                // treated as immediate.)
                let result = if op.params[0].resolved_value == op.params[1].resolved_value { 1 } else { 0 };
                self.check_address(op.params[2].unresolved_value as usize)?;
                self.memory[op.params[2].unresolved_value as usize] = result;
            },
            OperationTypes::RelativeBaseOffset => {
                // Increment the relative base by the value of parameter 0.
                self.relative_base = (self.relative_base as i64 + op.params[0].resolved_value) as usize;
            },
            OperationTypes::End => (),
        }

        Ok(())
    }

    // Makes sure the specified address is valid.  (For now, "valid" just means within the
    // bounds of the computer's memory.)
    fn check_address(&self, address: usize) -> Result<(), String> {
        if address >= self.memory.len() {
            return Err(format!(
                "Computer has insufficient memory! Requested address: {}",
                address
            ));
        }
        Ok(())
    }

    // Safely retrieve the data at a given memory address.
    fn fetch_from_address(&self, address: usize) -> Result<i64, String> {
        self.check_address(address).map(|_| self.memory[address])
    }
}

/// Helper function for running an Intcode program (from file) that can run all the way to
/// completion with a predetermined set of inputs (including no inputs).
pub fn load_and_run_computer(path: &str, inputs: &[i64]) -> Result<Vec<i64>, String> {
    let program = load_program(path).map_err(|e| format!("{:?}", e))?;
    run_computer(&program, inputs)
}

/// Helper function for running an Intcode program that can run all the way to
/// completion with a predetermined set of inputs (including no inputs).
pub fn run_computer(program: &[i64], inputs: &[i64]) -> Result<Vec<i64>, String> {
    let (in_sender, in_receiver) = mpsc::channel();
    let (out_sender, out_receiver) = mpsc::channel();

    for input in inputs {
        in_sender.send(*input).map_err(|e| format!("{:?}", e))?;
    }

    Computer::new(&program, in_receiver, out_sender).run()?;

    let mut outputs = Vec::new();
    while let Ok(output) = out_receiver.try_recv() {
        outputs.push(output);
    }
    Ok(outputs)
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
