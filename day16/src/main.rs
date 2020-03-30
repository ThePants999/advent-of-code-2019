#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]

use std::iter;

fn main() {
    let start_time = std::time::Instant::now();
    let signal = String::from("59755896917240436883590128801944128314960209697748772345812613779993681653921392130717892227131006192013685880745266526841332344702777305618883690373009336723473576156891364433286347884341961199051928996407043083548530093856815242033836083385939123450194798886212218010265373470007419214532232070451413688761272161702869979111131739824016812416524959294631126604590525290614379571194343492489744116326306020911208862544356883420805148475867290136336455908593094711599372850605375386612760951870928631855149794159903638892258493374678363533942710253713596745816693277358122032544598918296670821584532099850685820371134731741105889842092969953797293495");

    // Part 1 performs FFT over the full signal, though in an optimised way.
    // See `_old_part_1` and related code for a pretty but non-optimised alternative!
    let part_1_answer = full_fft(&signal, 100);

    //03036732577212944063491565474664
    //let part_2_answer = partial_fft("00000202577212944063491565474664", 1, 5);
    let part_2_answer = partial_fft(&signal, 10000, 100);

    // let offset: usize = signal[0..7].parse().unwrap();
    // let partial_fft = PartialFFT::new(signal, 10000).unwrap();
    // let part_2_answer = partial_fft
    //     .skip(99)
    //     .next()
    //     .unwrap()
    //     .iter()
    //     .skip(offset)
    //     .take(8)
    //     .map(std::string::ToString::to_string)
    //     .collect::<String>();
    println!(
        "Part 1: {}\nPart 2: {}\nTime: {}ms",
        part_1_answer,
        part_2_answer,
        start_time.elapsed().as_millis()
    );
}

fn full_fft(signal: &str, iterations: usize) -> String {
    let mut signal: Vec<i32> = signal
        .chars()
        .map(|c| c.to_digit(10).unwrap() as i32)
        .collect();
    
    for _ in 0..iterations {
        for output_index in 0..signal.len() {
            let output_position = output_index + 1;

            // There are three main optimisations we can make over a literal interpretation
            // of the FFT algorithm, courtesy of the nature of the repeating pattern.
            //
            // Firstly, because it's all zeroes and [minus] ones, we don't actually have
            // to do any multiplying. The value of each output element is simply the sum
            // of input elements that match ones in the pattern, minus the sum of input
            // elements that match minus ones in the pattern.
            //
            // Secondly, we don't actually need to construct the pattern - we just need to
            // iterate over the input elements, because we know that for the nth output
            // element, we want to take n input elements as positive, then ignore another
            // n, then take n of them as negative, then ignore another n, etc.
            //
            // Thirdly, because of the way the pattern starts with (n - 1) zeroes, we know
            // that we're starting at the nth input element. As well as making this better
            // than O(n^2), that also means that the set of output elements we've already
            // calculated match the set of input elements we don't care about - so we can
            // overwrite the input elements with the outputs.
            let mut tally = 0;
            let mut adding = true;
            let mut input_index = output_index;
            while input_index < signal.len() {
                for _ in 0..output_position {
                    if adding { tally += signal[input_index]; } else { tally -= signal[input_index]; }
                    input_index += 1;
                    if input_index == signal.len() { break; }
                }
                adding = !adding;
                input_index += output_position;
            }
            signal[output_index] = tally.abs() % 10;
        }
    }

    signal.iter().take(8).map(std::string::ToString::to_string).collect::<String>()
}

fn partial_fft(signal: &str, repeat: usize, iterations: usize) -> String {
    let offset: usize = signal[0..7].parse().unwrap();
    let full_length = signal.len() * repeat;
    assert!(offset > (full_length / 2));
    let mut signal: Vec<i32> = signal
        .chars()
        .cycle()
        .skip(offset)
        .take(full_length - offset)
        .map(|c| c.to_digit(10).unwrap() as i32)
        .collect();

    // More optimisations!
    //
    // The message we want to dig out of the output is more than half way through
    // the output.  (At least, it is with my inputs, and I suspect it's generically
    // true as it makes this optimisation possible!)  That's critical, because of
    // another feature of the repeating pattern.  Because the pattern for the nth
    // output element starts with (n - 1) zeroes and then n ones, then in a signal
    // of length k, if n > (k / 2), the output element is simply the sum of the last
    // (k - n + 1) input elements.  That is, the last output element is simply equal
    // to the last input element, the penultimate element equal to the sum of the
    // last two input elements, the one before that equal to the sum of the last
    // three, etc.
    // 
    // If you think about it, it gets better - that means that if we work backwards,
    // we need only keep a running tally and add one more input element each time,
    // so applying a phase of FFT becomes O(n).
    //
    // Finally, because each element depends on only elements after it, and we're
    // interested only in finding the result starting from `offset`, we can
    // completely ignore everything before `offset` (and have already achieved that
    // with the call to `skip` above).
    for _ in 0..iterations {
        let mut tally = 0;
        for index in (0..signal.len()).rev() {
            tally += signal[index];
            signal[index] = tally % 10;
        }
    }

    // Now we just want the first 8 digits again.
    signal.iter().take(8).map(std::string::ToString::to_string).collect::<String>()
}

// I've left in this first attempt at a solution because I think it's a really nice
// literal representation of the instructions in the question.  It isn't my final
// solution because it's not optimised at all, but shows how the words in the problem
// can be neatly translated to Rust if you don't think about how to do the same thing
// faster :-)
fn _old_part_1(input: &str) {
    let part_1_answer = FullFFT::new(input)
        .nth(100)
        .unwrap()
        .iter()
        .take(8)
        .map(std::string::ToString::to_string)
        .collect::<String>();
    println!("Part 1: {}", part_1_answer);
}

struct FullFFT {
    current_value: Vec<i32>,
}

#[allow(dead_code)]
impl FullFFT {
    fn new(input: &str) -> Self {
        Self {
            current_value: input
                .chars()
                .map(|c| c.to_digit(10).unwrap() as i32)
                .collect(),
        }
    }
}

impl Iterator for FullFFT {
    type Item = Vec<i32>;
    fn next(&mut self) -> Option<Self::Item> {
        self.current_value = (1..=self.current_value.len())
            .map(|index| {
                fft_pattern(index)
                    .zip(self.current_value.iter())
                    .map(|(pattern, digit)| pattern * *digit)
                    .sum::<i32>()
                    .abs()
                    % 10
            })
            .collect();
        Some(self.current_value.clone())
    }
}

// This builds the pattern that would apply to the element in a given position.
fn fft_pattern(position: usize) -> impl Iterator<Item = i32> {
    iter::repeat(0)
        .take(position)
        .chain(iter::repeat(1).take(position))
        .chain(iter::repeat(0).take(position))
        .chain(iter::repeat(-1).take(position))
        .cycle()
        .skip(1)
}
