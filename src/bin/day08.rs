use lib::error::Fail;
use lib::input::{read_file_as_string, run_with_input};
use std::collections::HashMap;

use ndarray::prelude::*;

#[derive(Debug)]
enum BadInput {
    Incomplete(String),
}

impl From<BadInput> for Fail {
    fn from(e: BadInput) -> Fail {
        match e {
            BadInput::Incomplete(msg) => Fail(format!("bad input: input is incomplete: {}", msg)),
        }
    }
}

fn parse_input(w: usize, h: usize, input_body: String) -> Result<Vec<Array2<char>>, BadInput> {
    let input: Vec<char> = input_body.trim().chars().collect();
    let mut result = Vec::new();
    let pixels_per_layer = w * h;
    if input.len() % pixels_per_layer != 0 {
        return Err(BadInput::Incomplete(format!(
            "{} pixels is not enough to fill a whole number of {}x{} layers",
            input.len(),
            w,
            h
        )));
    }
    let total_layers = input.len() / pixels_per_layer;
    while result.len() < total_layers {
        let layer_base_pix_pos = result.len() * pixels_per_layer;
        result.push(Array::from_shape_fn((h, w), |(r, c)| {
            let pixpos = layer_base_pix_pos + (r * w) + c;
            input[pixpos]
        }))
    }
    Ok(result)
}

fn layer_popcounts(layers: &[Array2<char>]) -> HashMap<usize, HashMap<char, usize>> {
    let mut result: HashMap<usize, HashMap<char, usize>> = HashMap::new();
    for (layer_number, layer) in layers.iter().enumerate() {
        let entry = result.entry(layer_number).or_insert_with(HashMap::new);
        for ch in layer.iter() {
            *entry.entry(*ch).or_insert(0) += 1;
        }
    }
    result
}

fn part1(layers: &[Array2<char>]) {
    let popcounts = layer_popcounts(layers);
    let layer_with_fewest_zeroes: usize = popcounts
        .iter()
        .map(|(layer_num, counts_by_char)| {
            let zeroes: usize = counts_by_char[&'0'];
            (zeroes, *layer_num)
        })
        .min()
        .unwrap()
        .1;
    let layercounts = popcounts.get(&layer_with_fewest_zeroes).unwrap();
    let result = layercounts[&'1'] * layercounts[&'2'];
    println!("Day 8 part 1: {}", result);
}

fn part2(layers: &[Array2<char>], w: usize, h: usize) {
    for row in 0..h {
        for col in 0..w {
            let pos = (row, col);
            let ch: Option<char> = layers.iter().map(|layer| layer[pos]).find(|ch| *ch != '2');
            match ch {
                Some('1') => {
                    print!("#"); // white
                }
                Some('0') => {
                    print!(" "); // black
                }
                None => {
                    print!("."); // transparent
                }
                Some(c) => {
                    panic!("pixel colour is {}", c);
                }
            }
        }
        println!();
    }
}

const WIDTH: usize = 25;
const HEIGHT: usize = 6;

fn run(input: String) -> Result<(), Fail> {
    let layers: Vec<Array2<char>> = parse_input(WIDTH, HEIGHT, input)?;
    println!("We have {} layers", layers.len());
    part1(&layers);
    part2(&layers, WIDTH, HEIGHT);
    Ok(())
}

fn main() -> Result<(), Fail> {
    run_with_input(8, read_file_as_string, run)
}
