use std::collections::HashMap;

use ndarray::prelude::*;

use aoc::read_stdin_as_string;

#[derive(Debug)]
enum BadInput {
    IOError(std::io::Error),
    Incomplete(String),
}

fn parse_input(w: usize, h: usize) -> Result<Vec<Array2<char>>, BadInput> {
    let input: Vec<char> = read_stdin_as_string()
        .map_err(BadInput::IOError)?
        .trim()
        .chars()
        .collect();
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

#[derive(Debug)]
enum Fail {
    BadInput(BadInput),
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

fn main() -> Result<(), Fail> {
    const WIDTH: usize = 25;
    const HEIGHT: usize = 6;
    let layers = parse_input(WIDTH, HEIGHT).map_err(Fail::BadInput)?;
    println!("We have {} layers", layers.len());
    part1(&layers);
    part2(&layers, WIDTH, HEIGHT);
    Ok(())
}
