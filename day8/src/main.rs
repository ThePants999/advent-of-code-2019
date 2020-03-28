use std::io;
use std::io::Read;
use std::fs::File;
use std::process;

fn main() {
    let width = 25;
    let height = 6;

    let image = load_image(width, height).unwrap_or_else(|err| {
        println!("Could not load input file!\n{:?}", err);
        process::exit(1);
    });

    let mut layers_data: Vec<[i32; 3]> = Vec::new();
    for layer in &image.layers {
        let mut counts: [i32; 3] = [0; 3];
        for pixel in layer {
            match pixel {
                0 | 1 | 2 => counts[*pixel as usize] += 1,
                _ => (),
            }
        }
        layers_data.push(counts);
    }

    let mut lowest_zeroes = layers_data[0][0];
    let mut score = layers_data[0][1] * layers_data[0][2];
    for layer_data in layers_data {
        if layer_data[0] < lowest_zeroes {
            lowest_zeroes = layer_data[0];
            score = layer_data[1] * layer_data[2];
        }
    }

    let mut picture = String::new();
    for i in 0..(width * height) {
        if i % width == 0 { picture.push('\n'); }
        for layer in &image.layers {
            match layer[i] {
                0 => { picture.push(' '); break; },
                1 => { picture.push('*'); break; },
                _ => ()
            }
        }        
    }

    println!("{}{}", score, picture);
}

struct Image {
    width: usize,
    height: usize,
    layers: Vec<Vec<i32>>,
}

impl Image {
    fn new(width: usize, height: usize) -> Self {
        let mut layers = Vec::new();
        layers.push(Vec::with_capacity(width * height));
        Self {
            width: width,
            height: height,
            layers: layers,
        }
    }

    fn new_layer(&mut self) {
        self.layers.push(Vec::with_capacity(self.width * self.height));
    }

    fn add_pixel(&mut self, pixel: i32) {
        if self.layers.last().unwrap().len() == (self.width * self.height) { self.new_layer(); }
        self.layers.last_mut().unwrap().push(pixel);
    }
}

fn load_image(width: usize, height: usize) -> Result<Image, io::Error> {
    let mut input = File::open("day8/input.txt")?;
    let mut data = String::new();
    input.read_to_string(&mut data)?;

    let mut image = Image::new(width, height);
    data.chars().map(|c| c.to_digit(10).unwrap()).for_each(|digit| {
    //ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, format!("Invalid character found in input data: {}", c)))?
        image.add_pixel(digit as i32);
    });

    Ok(image)
}