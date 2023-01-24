use std::collections::VecDeque;
use itertools::iproduct;

use crate::lang::*;

pub fn synthesize(examples: &[(StringExpr, StringExpr)]) -> StringExpr {
    bottom_up(examples).find(|exp| {
        examples.iter().all(|(inp, out)| {
            exp.clone().simplify(inp) == out.clone()
        })
    }).unwrap()
}

pub fn bottom_up(examples: &[(StringExpr, StringExpr)]) -> impl Iterator<Item = StringExpr> + '_ {
    use StringExpr::*;

    let mut i = 0;
    let mut bank = vec![
        Loc(0),
        Lit(" ".to_string()),
        Input,
    ];

    std::iter::from_fn(move || {
        if i == bank.len() {
            let mut adjs = {
                let strings = bank.iter().filter(|e| matches!(e, Lit(_) | Concat{..} | Slice{..} | Input));
                let locs = bank.iter().filter(|e| matches!(e, Loc(_) | Index{..}));

                let concats = iproduct!(strings.clone(), strings.clone()).map(|(lhs, rhs)| {
                    Concat { lhs: Box::new(lhs.clone()), rhs: Box::new(rhs.clone()) }
                });

                let indexes = iproduct!(strings.clone(), strings.clone()).map(|(outer, inner)| {
                    Index { outer: Box::new(outer.clone()), inner: Box::new(inner.clone()) }
                });

                let slices = iproduct!(strings.clone(), locs.clone(), locs.clone()).map(|(outer, start, end)| {
                    Slice { outer: Box::new(outer.clone()), start: Box::new(start.clone()), end: Box::new(end.clone()) }
                });

                concats.chain(indexes).chain(slices)
            }.collect::<Vec<_>>();

            bank.append(&mut adjs);
        }

        i += 1;
        Some(bank[i - 1].clone())
    })
}
