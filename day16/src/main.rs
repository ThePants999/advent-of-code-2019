use std::iter;

fn main() {
    let signal = String::from("59755896917240436883590128801944128314960209697748772345812613779993681653921392130717892227131006192013685880745266526841332344702777305618883690373009336723473576156891364433286347884341961199051928996407043083548530093856815242033836083385939123450194798886212218010265373470007419214532232070451413688761272161702869979111131739824016812416524959294631126604590525290614379571194343492489744116326306020911208862544356883420805148475867290136336455908593094711599372850605375386612760951870928631855149794159903638892258493374678363533942710253713596745816693277358122032544598918296670821584532099850685820371134731741105889842092969953797293495");
    let offset = signal[0..7].parse::<usize>().unwrap();
    let mut fft = PartialFFT::new(signal, 10000).unwrap();
    println!("Starting");
    for i in 1..100 {
        fft.next();
        println!("Progress: {}%", i);
    }
    let final_value = fft.next().unwrap().iter().map(|digit| digit.to_string()).collect::<String>();
    let result = &final_value[offset..offset + 8];
    println!("Offset: {}\nResult: {}", offset, result);
}

struct PartialFFT {
    current_value: Vec<i32>,
    extra_digits: usize,
}

impl PartialFFT {
    fn new(signal: String, repetition: usize) -> Result<Self, String> {
        let signal_len = signal.len();
        let mut fft = Self {
            current_value: iter::repeat(signal)
                .take(repetition - (repetition / 2)) // Half of repetitions rounded up
                .collect::<String>()
                .chars()
                .map(|c| {
                    c.to_digit(10)
                        .map(|i| i as i32)
                        .ok_or(format!("Invalid character in input: {}", c))
                })
                .collect::<Result<Vec<i32>, String>>()?,
            extra_digits: signal_len * (repetition / 2),
        };
        fft.current_value.reverse();
        Ok(fft)
    }
}

impl Iterator for PartialFFT {
    type Item = Vec<i32>;

    fn next(&mut self) -> Option<Self::Item> {
        let inner = PartialFFTInner::new(&self.current_value);
        self.current_value = inner.collect();
        let mut extra_digits: Vec<i32> = iter::repeat(0 as i32).take(self.extra_digits).collect();
        let mut return_value = self.current_value.clone();
        return_value.append(&mut extra_digits);
        return_value.reverse();
        Some(return_value)
    }
}

struct PartialFFTInner {
    previous_digits: Vec<i32>,
    running_total: i64,
}
 
impl PartialFFTInner {
    fn new(input_digits: &Vec<i32>) -> Self {
        let mut previous_digits = input_digits.clone();
        previous_digits.reverse();
        Self {
            previous_digits: previous_digits,
            running_total: 0,
        }
    }
}

impl Iterator for PartialFFTInner {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        self.previous_digits.pop().map(|digit| {
            self.running_total += digit as i64;
            (self.running_total % 10) as i32
        })
    }
}

// struct FullFFT {
//     current_value: Vec<i32>,
// }

// impl FullFFT {
//     fn new(input: String) -> Result<Self, String> {
//         Ok(Self {
//             current_value: input
//                 .chars()
//                 .map(|c| {
//                     c.to_digit(10)
//                         .map(|i| i as i32)
//                         .ok_or(format!("Invalid character in input: {}", c))
//                 })
//                 .collect::<Result<Vec<i32>, String>>()?,
//         })
//     }
// }

// impl Iterator for FullFFT {
//     type Item = Vec<i32>;
//     fn next(&mut self) -> Option<Self::Item> {
//         self.current_value = (1..=self.current_value.len())
//             .map(|index| {
//                 fft_pattern(index)
//                     .zip(self.current_value.iter())
//                     .map(|(pattern, digit)| pattern * *digit)
//                     .sum::<i32>()
//                     .abs()
//                     % 10
//             })
//             .collect();
//         Some(self.current_value.clone())
//     }
// }

// fn fft_pattern(position: usize) -> impl Iterator<Item = i32> {
//     iter::repeat(0)
//         .take(position)
//         .chain(iter::repeat(1).take(position))
//         .chain(iter::repeat(0).take(position))
//         .chain(iter::repeat(-1).take(position))
//         .cycle()
//         .skip(1)
// }
