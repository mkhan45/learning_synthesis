use itertools::iproduct;
use std::collections::HashSet;

use crate::vsa::{Fun, Lit, AST};

type Prog = AST<Lit, Fun>;
type VSA = crate::vsa::VSA<Lit, Fun>;

pub fn bottom_up(examples: &[(Lit, Lit)]) -> Option<Prog> {
    let mut bank = &VSA::default();

    for _ in 0..10 {
        let adjs = todo!();
        // let adjs: Vec<Prog> = {
        //     todo!()
        // }.collect();

        // for adj in adjs {
        //     if examples.iter().all(|(inp, out)| &adj.eval(inp) == out) {
        //         return Some(adj);
        //     }
        // }
    }

    None
}
