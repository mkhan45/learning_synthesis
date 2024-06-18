#![cfg(test)]

use crate::datagen::*;

use std::fs::File;
use std::path::Path;
use std::io::{self, BufRead};

const TEST: StringRNGToken = StringRNGToken::Test;

// generates input strings
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

// infinite program iterator
#[test]
fn test_prog_iterator() {
    let bank = vec![
        AST::Lit(Lit::LocConst(0)),
        AST::Lit(Lit::LocConst(1)),
        AST::Lit(Lit::LocConst(2)),
        AST::Lit(Lit::LocConst(3)),
    ];
    let ops = vec![
        vec![],
        vec![Fun::LocAdd, Fun::LocSub],
    ];
    let mut gen = ProgramGen::new(bank, ops);
    dbg!(gen.nth(10_000_000));
}
