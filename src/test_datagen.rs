#![cfg(test)]

use crate::datagen::*;

use std::fs::File;
use std::io::{self, BufRead};

const TEST: StringRNGToken = StringRNGToken::Test;

#[test]
fn test_datagen() {
    let file = File::open("data/words.txt").unwrap();
    let dict: Vec<String> = io::BufReader::new(file)
        .lines()
        .map(|line| line.unwrap())
        .collect();

    let mut string_rng: StringRNG<TEST> = 
        StringRNG::new(dict, vec![".".to_string(), ",".to_string(), ":".to_string()]);

    dbg!(string_rng.gen_string_fr(4));
    dbg!(string_rng.gen_string_fr(5));
    dbg!(string_rng.gen_string_fr(6));
    dbg!(string_rng.gen_string_fr(7));
}
