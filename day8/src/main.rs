#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]

use std::io::{self, Read};

const WIDTH: usize = 25;
const HEIGHT: usize = 6;
const PIXELS_PER_LAYER: usize = WIDTH * HEIGHT;

fn main() {
    let start_time = std::time::Instant::now();

    let image = load_image().unwrap_or_else(|err| {
        println!("Could not load input file!\n{:?}", err);
        std::process::exit(1);
    });

    let max_zeroes = image.layers.iter().map(Layer::num_zeroes).min().unwrap();
    let part_1_score = image.layers.iter().find(|layer| layer.num_zeroes() == max_zeroes).unwrap().part_1_score();
    let part_2_picture = image.draw();

    println!(
        "Part 1: {}\nPart 2: {}\nTime: {}ms",
        part_1_score,
        part_2_picture,
        start_time.elapsed().as_millis()
    );
}

struct Layer {
    pixels: Vec<usize>,
    pixel_counts: [usize; 3],
}

impl Layer {
    fn new() -> Self {
        Self {
            pixels: Vec::with_capacity(PIXELS_PER_LAYER),
            pixel_counts: [0; 3],
        }
    }

    fn add_pixel(&mut self, pixel: usize) {
        self.pixels.push(pixel);
        self.pixel_counts[pixel] += 1;
    }

    fn is_full(&self) -> bool {
        self.pixels.len() == PIXELS_PER_LAYER
    }

    fn num_zeroes(&self) -> usize {
        self.pixel_counts[0]
    }

    fn part_1_score(&self) -> usize {
        self.pixel_counts[1] * self.pixel_counts[2]
    }
}

struct Image {
    layers: Vec<Layer>,
}

impl Image {
    fn from_str(data: &str) -> Self {
        let mut image = Self { layers: Vec::new() };
        image.new_layer();

        data.chars()
            .map(|c| c.to_digit(10).unwrap())
            .for_each(|digit| {
                image.add_pixel(digit as usize);
            });

        image
    }

    fn new_layer(&mut self) {
        self.layers.push(Layer::new());
    }

    fn add_pixel(&mut self, pixel: usize) {
        if self.layers.last().unwrap().is_full() { self.new_layer(); }
        self.layers.last_mut().unwrap().add_pixel(pixel);
    }

    fn draw(&self) -> String {
        let mut picture = String::new();

        for i in 0..PIXELS_PER_LAYER {
            if i % WIDTH == 0 {
                picture.push('\n');
            }
            for layer in &self.layers {
                match layer.pixels[i] {
                    0 => {
                        picture.push(' ');
                        break;
                    }
                    1 => {
                        picture.push('#');
                        break;
                    }
                    _ => (),
                }
            }
        }

        picture
    }
}

fn load_image() -> Result<Image, io::Error> {
    let mut input = std::fs::File::open("day8/input.txt")?;
    let mut data = String::new();
    input.read_to_string(&mut data)?;

    Ok(Image::from_str(&data))
}
