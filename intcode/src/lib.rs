//! # Intcode
//!
//! `intcode` is a library for executing Intcode programs, as featured in the Advent
//! of Code 2019.
//!
//! This library offers three modes of operation, represented by the structs
//! [`ChannelIOComputer`], [`StreamingIOComputer`] and [`SynchronousComputer`].  All
//! three take an Intcode program and will execute it, but differ in how they do so.
//!
//! [`ChannelIOComputer`] is designed to run in its own thread, and communicates with
//! your main thread using MPSC channels.  It will run continuously, and block waiting
//! for input to be sent on its inbound channel if there isn't any waiting when the
//! Intcode program requests some, with outputs being sent back on its outbound channel.
//!
//! [`StreamingIOComputer`] is designed to run in a Tokio reactor, and communicates with
//! your code using Tokio MPSC streams.  It works with async code, and will run
//! continuously but lazily as required by your `await` calls.  `await` will block
//! forever if the computer can't get enough inputs, so make sure that you've either
//! sent enough on the inbound stream before `await`ing, or you've got appropriate
//! futures chaining so that enough inputs will be sent during execution.
//!
//! [`SynchronousComputer`] is designed to run on your main thread, and while you can
//! provide it a set of inputs when you execute it, if it needs more after consuming
//! those, it will return, and you'll need to call it again with further input(s).
//!
//! Helper functions assist with loading programs from file, and executing them via
//! [`ChannelIOComputer`]s or [`StreamingIOComputer`]s.
//!
//! Typical usage might look like this:
//!
//! ```
//! let program = intcode::load_program("path/to/input.txt").unwrap_or_else(|err| {
//!     println!("Could not load input file!\n{:?}", err);
//!     process::exit(1);
//! });
//!
//! // ChannelIOComputer
//! let (in_send, in_recv) = std::sync::mpsc::channel();
//! let (out_send, out_recv) = std::sync::mpsc::channel();
//! let mut channel_comp = intcode::ChannelIOComputer::new(&program, in_recv, out_send);
//! std::thread::spawn(move || {
//!     channel_comp.run().unwrap_or_else(|e| {
//!         println!("Computer failed: {}", e);
//!         process::exit(1);
//!     });
//! });
//!
//! // The computer is now executing in parallel, and you can communicate with it
//! // via its channels.
//! in_send.send(1).unwrap();
//! println!("{}", out_recv.recv().unwrap());
//!
//! // StreamingIOComputer
//! let (in_send, in_recv) = tokio::sync::mpsc::unbounded_channel();
//! let (out_send, out_recv) = tokio::sync::mpsc::unbounded_channel();
//! tokio::spawn(async move {
//!     intcode::StreamingIOComputer::new(&program, in_recv, out_send).run().await;
//! });
//!
//! // The computer is now executing on the Tokio runtime, and you can communicate
//! // with it via its streams.
//! in_send.send(1).unwrap();
//! println!("{}", out_recv.recv().await.unwrap());
//!
//! // SynchronousComputer
//! let mut sync_comp = intcode::SynchronousComputer::new(&program);
//! let output = sync_comp.run(&[1]);
//! println!("{}", output.outputs.first().unwrap());
//! ```
//!
//! [`ChannelIOComputer`]: ./struct.ChannelIOComputer.html
//! [`StreamingIOComputer`]: ./struct.StreamingIOComputer.html
//! [`SynchronousComputer`]: ./struct.SynchronousComputer.html
//!

#![crate_name = "intcode"]
#![crate_type = "lib"]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]

use std::collections::VecDeque;
use std::fs::File;
use std::io;
use std::io::Read;
use std::iter::FromIterator;
use std::sync::mpsc::{self, Receiver, Sender};

extern crate tokio;
use tokio::stream::StreamExt;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

/// The result of running a `SynchronousComputer` as far as possible.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SynchronousComputeResult {
    /// The program has run to completion.
    ProgramEnded,

    /// The program has paused until you provide further input.
    InputRequired,
}

/// Output from running a `SynchronousComputer` as far as possible.
pub struct SynchronousComputeOutput {
    /// The reason why execution has stopped.
    pub result: SynchronousComputeResult,

    /// The outputs that have been generated during this round of execution.
    pub outputs: Vec<i64>,
}

/// A virtual computer whose memory contains an Intcode program and which can execute
/// said program, where communication with the computer is via synchronous function
/// calls.
pub struct SynchronousComputer {
    processor: Processor,
    last_result: Option<SynchronousComputeResult>,
}

impl SynchronousComputer {
    /// Construct a computer to run `program`.  `program` only needs to be as long as the
    /// instructions and data contained within it; the computer has additional memory available
    /// that the program can refer to.
    #[must_use]
    pub fn new(program: &[i64]) -> Self {
        let processor = Processor::new(program);
        Self {
            processor,
            last_result: None,
        }
    }

    /// Executes the program in the computer's memory as far as possible, returning either when
    /// the program completes or if an input is required when all the provided inputs have been
    /// used up.
    ///
    /// # Panics
    ///
    /// Panics if any problem is hit executing the program, which would indicate either that the
    /// program is invalid, or that invalid inputs were provided to it.
    pub fn run(&mut self, inputs: &[i64]) -> SynchronousComputeOutput {
        let mut inputs = VecDeque::from_iter(inputs);
        let mut outputs = Vec::new();

        match self.last_result {
            Some(SynchronousComputeResult::ProgramEnded) => {
                return SynchronousComputeOutput {
                    result: SynchronousComputeResult::ProgramEnded,
                    outputs,
                }
            }
            Some(SynchronousComputeResult::InputRequired) if inputs.is_empty() => {
                return SynchronousComputeOutput {
                    result: SynchronousComputeResult::InputRequired,
                    outputs,
                }
            }
            Some(SynchronousComputeResult::InputRequired) => {
                self.processor.input_available(*inputs.pop_front().unwrap())
            }
            _ => (),
        }

        let output = loop {
            match self.processor.process() {
                SingleOperationResult::Handled => (),
                SingleOperationResult::InputRequired => {
                    if let Some(input) = inputs.pop_front() {
                        self.processor.input_available(*input);
                    } else {
                        break SynchronousComputeOutput {
                            result: SynchronousComputeResult::InputRequired,
                            outputs,
                        };
                    }
                }
                SingleOperationResult::OutputAvailable(output) => outputs.push(output),
                SingleOperationResult::ProgramEnded => {
                    break SynchronousComputeOutput {
                        result: SynchronousComputeResult::ProgramEnded,
                        outputs,
                    }
                }
            }
        };

        self.last_result = Some(output.result);
        output
    }
}

/// `StreamingIOComputer`s send these on their output stream to keep you informed of
/// what's happening in the computer.
#[derive(Debug)]
pub enum AsyncComputeNotification {
    /// The computer has generated an output.
    Output(i64),

    /// The computer has paused waiting for input.  (Of course, with the nature of async
    /// code, if you've sent one recently, it may satisfy this demand!)
    InputRequired,

    /// The program has reached a successful conclusion.
    ProgramEnded,
}

/// A virtual computer whose memory contains an Intcode program and which can execute
/// said program, where communication with the computer is via Tokio MPSC streams, and
/// the computer is intended to be executed on a Tokio reactor.
pub struct StreamingIOComputer {
    processor: Processor,
    in_stream: UnboundedReceiver<i64>,
    out_stream: UnboundedSender<AsyncComputeNotification>,
}

impl StreamingIOComputer {
    /// Construct a computer to run `program`.  `program` only needs to be as long as the
    /// instructions and data contained within it; the computer has additional memory available
    /// that the program can refer to.
    ///
    /// `in_channel` is the receive half of a stream on which you can send runtime inputs to
    /// the program, and `out_channel` is the send half of a stream on which you can receive
    /// runtime outputs.
    #[must_use]
    pub fn new(
        program: &[i64],
        in_stream: UnboundedReceiver<i64>,
        out_stream: UnboundedSender<AsyncComputeNotification>,
    ) -> Self {
        Self {
            processor: Processor::new(program),
            in_stream,
            out_stream,
        }
    }

    /// Executes the program in the computer's memory.
    ///
    /// This is an async function which will run the program through to completion, but may
    /// `await` input on the inbound stream if sufficient inputs are not pre-sent.  Typically
    /// you would spawn this function onto a Tokio reactor.
    ///
    /// # Panics
    ///
    /// Panics if any problem is hit executing the program, which would indicate either that the
    /// program is invalid, or that invalid inputs were provided to it, or that the streams were
    /// closed prematurely.
    pub async fn run(&mut self) {
        loop {
            match self.processor.process() {
                SingleOperationResult::Handled => (),
                SingleOperationResult::InputRequired => {
                    self.out_stream
                        .send(AsyncComputeNotification::InputRequired)
                        .expect("Intcode computer tried to send an output but channel was closed!");
                    let input = self
                        .in_stream
                        .next()
                        .await
                        .expect("Intcode computer expected an input but channel was closed!");
                    self.processor.input_available(input);
                }
                SingleOperationResult::OutputAvailable(output) => self
                    .out_stream
                    .send(AsyncComputeNotification::Output(output))
                    .expect("Intcode computer tried to send an output but channel was closed!"),
                SingleOperationResult::ProgramEnded => {
                    self.out_stream
                        .send(AsyncComputeNotification::ProgramEnded)
                        .expect("Intcode computer tried to send an output but channel was closed!");
                    break;
                }
            }
        }
    }
}

/// A virtual computer whose memory contains an Intcode program and which can execute
/// said program, where communication with the computer is via MPSC channels, and the
/// computer is intended to be executed on its own thread.
pub struct ChannelIOComputer {
    processor: Processor,
    in_channel: Receiver<i64>,
    out_channel: Sender<i64>,
}

impl ChannelIOComputer {
    /// Construct a computer to run `program`.  `program` only needs to be as long as the
    /// instructions and data contained within it; the computer has additional memory available
    /// that the program can refer to.
    ///
    /// `in_channel` is the receive half of a channel on which you can send runtime inputs to
    /// the program, and `out_channel` is the send half of a channel on which you can receive
    /// runtime outputs.
    pub fn new(program: &[i64], in_channel: Receiver<i64>, out_channel: Sender<i64>) -> Self {
        let processor = Processor::new(program);
        Self {
            processor,
            in_channel,
            out_channel,
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
            match self.processor.process() {
                SingleOperationResult::Handled => (),
                SingleOperationResult::InputRequired => {
                    let input = self
                        .in_channel
                        .recv()
                        .expect("Intcode computer expected an input but channel was closed!");
                    self.processor.input_available(input);
                }
                SingleOperationResult::OutputAvailable(output) => self
                    .out_channel
                    .send(output)
                    .expect("Intcode computer tried to send an output but channel was closed!"),
                SingleOperationResult::ProgramEnded => break,
            }
        }
    }

    /// Returns the value currently stored at memory address zero.
    ///
    /// This function shouldn't exist - Intcode programs should output anything that users
    /// might need to get their hands on.  It exists solely for the purposes of day 2, where
    /// we needed to get an output from the computer before the Output instruction was
    /// introduced.  The fact that it specifically fetches address zero, rather than being
    /// a generic function to read the computer's memory, is to reinforce that you're not
    /// supposed to use it - the computer's memory is supposed to be private, and Input and
    /// Output instructions are the communication channel.
    pub fn fetch_address_zero(&self) -> i64 {
        self.processor.memory[0]
    }
}

// The result of executing a single operation on a computer.
enum SingleOperationResult {
    Handled,
    ProgramEnded,
    InputRequired,
    OutputAvailable(i64),
}

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

struct Processor {
    memory: Vec<i64>,
    instruction_pointer: i64,
    relative_base: i64,
    input_location: Option<i64>,
    stored_inputs: VecDeque<i64>,
}

impl Processor {
    pub fn new(program: &[i64]) -> Self {
        Self {
            memory: program.into(),
            instruction_pointer: 0,
            relative_base: 0,
            input_location: None,
            stored_inputs: VecDeque::new(),
        }
    }

    // --- Interface to Computer ---

    fn process(&mut self) -> SingleOperationResult {
        let operation = self.fetch_operation();
        self.execute_operation(&operation)
    }

    fn input_available(&mut self, input: i64) {
        if let Some(location) = self.input_location {
            // We've previously evaluated an Input operation when we had no input available,
            // so execute that operation with this input.
            self.set_at_address(location, input);
            self.input_location = None;
        } else {
            // The program hasn't requested this input yet, so store it.
            self.stored_inputs.push_back(input);
        }
    }

    // --- Private methods ---

    // Execute a single operation that's been fully parsed from memory and requires no I/O.
    // -  Input should be fetched before calling this function, and stored in `op`.
    // -  Output should be handled entirely without this function.
    fn execute_operation(&mut self, op: &Operation) -> SingleOperationResult {
        match op.optype {
            OperationType::Add => {
                self.set_at_address(op.params.c, op.params.a + op.params.b);
                SingleOperationResult::Handled
            }
            OperationType::Multiply => {
                self.set_at_address(op.params.c, op.params.a * op.params.b);
                SingleOperationResult::Handled
            }
            OperationType::Input => {
                if let Some(input) = self.stored_inputs.pop_front() {
                    self.set_at_address(op.params.a, input);
                    SingleOperationResult::Handled
                } else {
                    assert!(self.input_location.is_none());
                    self.input_location = Some(op.params.a);
                    SingleOperationResult::InputRequired
                }
            }
            OperationType::Output => SingleOperationResult::OutputAvailable(op.params.a),
            OperationType::JumpIfTrue => {
                if op.params.a != 0 {
                    self.instruction_pointer = op.params.b;
                }
                SingleOperationResult::Handled
            }
            OperationType::JumpIfFalse => {
                if op.params.a == 0 {
                    self.instruction_pointer = op.params.b;
                }
                SingleOperationResult::Handled
            }
            OperationType::LessThan => {
                self.set_at_address(op.params.c, op.params.a < op.params.b);
                SingleOperationResult::Handled
            }
            OperationType::Equals => {
                self.set_at_address(op.params.c, op.params.a == op.params.b);
                SingleOperationResult::Handled
            }
            OperationType::RelativeBaseOffset => {
                self.relative_base += op.params.a;
                SingleOperationResult::Handled
            }
            OperationType::End => SingleOperationResult::ProgramEnded,
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
/// completion with a predetermined set of inputs (including no inputs), on its own thread.
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
    Ok(run_parallel_computer(&program, inputs))
}

/// Helper function for running an Intcode program that can run all the way to
/// completion with a predetermined set of inputs (including no inputs), on its own
/// thread.
///
/// # Panics
///
/// Panics if the supplied program is invalid.
#[allow(clippy::must_use_candidate)]
pub fn run_parallel_computer(program: &[i64], inputs: &[i64]) -> Vec<i64> {
    let (in_sender, in_receiver) = mpsc::channel();
    let (out_sender, out_receiver) = mpsc::channel();

    for input in inputs {
        in_sender.send(*input).unwrap();
    }

    ChannelIOComputer::new(&program, in_receiver, out_sender).run();

    let mut outputs = Vec::new();
    while let Ok(output) = out_receiver.try_recv() {
        outputs.push(output);
    }
    outputs
}

/// Helper function for running an Intcode program that can run all the way to
/// completion with a predetermined set of inputs (including no inputs), on a
/// futures executor (e.g. a Tokio reactor).
///
/// # Panics
///
/// Panics if the supplied program is invalid, or if insufficient inputs were
/// provided to run it to completion.
pub async fn run_async_computer(program: &[i64], inputs: &[i64]) -> Vec<i64> {
    let (in_sender, in_receiver) = unbounded_channel();
    let (out_sender, out_receiver) = unbounded_channel();

    for input in inputs {
        in_sender.send(*input).unwrap();
    }

    StreamingIOComputer::new(&program, in_receiver, out_sender)
        .run()
        .await;

    out_receiver.filter_map(|notification| match notification {
        AsyncComputeNotification::InputRequired => panic!("Insufficient inputs given to program!"),
        AsyncComputeNotification::ProgramEnded => None,
        AsyncComputeNotification::Output(output) => Some(output),
    }).collect().await
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
