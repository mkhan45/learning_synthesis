use std::collections::VecDeque;
use crate::lang::*;
use itertools::iproduct;

// Duet
// https://dl.acm.org/doi/10.1145/3434335

type Bank = Vec<(StringExpr, Vec<StringExpr>)>;

pub fn combo_synth(examples: &[(StringExpr, StringExpr)]) -> Option<StringExpr> {
    let mut bank: Bank = Vec::new();
    let mut add_to_bank = |prog: StringExpr| {
        let simpl = prog.simplify(&StringExpr::Input);
        let outs = examples
            .iter()
            .map(|(inp, _)| simpl.simplify(inp))
            .collect::<Vec<_>>();
        bank.push((simpl, outs));
    };
    add_to_bank(StringExpr::Loc(Some(0)));
    add_to_bank(StringExpr::Loc(Some(1)));
    add_to_bank(StringExpr::Loc(None));
    add_to_bank(StringExpr::Lit(" ".to_string()));
    add_to_bank(StringExpr::Input);

    let mut size = 1;
    loop {
        bottom_up(&mut bank, examples, size);
        size += 1;
        if let r@Some(_) = top_down(&bank, examples) {
            return r;
        }
    }
}

// just make the bank
pub fn bottom_up(bank: &mut Bank, examples: &[(StringExpr, StringExpr)], max_size: usize) {
    use StringExpr::*;

    loop {
        let mut too_big = false;
        // TODO: convert to normal for loop :(
        let adjs = {
            let strings = bank
                .iter()
                .map(|(e, _)| e)
                .filter(|e| matches!(e, Lit(_) | Concat { .. } | Slice { .. } | Input));

            let locs = bank
                .iter()
                .map(|(e, _)| e)
                .filter(|e| matches!(e, Loc(_) | Index { .. }));

            let loc_adds = iproduct!(locs.clone(), locs.clone()).map(|(lhs, rhs)| LocAdd {
                lhs: Box::new(lhs.clone()),
                rhs: Box::new(rhs.clone()),
            });

            let concats = iproduct!(strings.clone(), strings.clone()).map(|(lhs, rhs)| Concat {
                lhs: Box::new(lhs.clone()),
                rhs: Box::new(rhs.clone()),
            });

            let indexes = iproduct!(strings.clone(), strings.clone()).map(|(outer, inner)| Index {
                outer: Box::new(outer.clone()),
                inner: Box::new(inner.clone()),
            });

            let slices = iproduct!(strings.clone(), locs.clone(), locs.clone()).map(
                |(outer, start, end)| Slice {
                    outer: Box::new(outer.clone()),
                    start: Box::new(start.clone()),
                    end: Box::new(end.clone()),
                },
            );

            slices.chain(concats).chain(indexes).chain(loc_adds)
        }
        .map(|prog| {
            if !too_big && prog.size() >= max_size {
                too_big = true;
            }

            prog.simplify(&StringExpr::Input)
        })
        .collect::<Vec<_>>();

        if too_big {
            dbg!();
            return;
        }

        dbg!(adjs.len());
        for adj in adjs {
            let redundant = bank.iter().any(|(_, bank_outs)| {
                bank_outs
                    .iter()
                    .zip(examples.iter())
                    .all(|(bank_out, (inp, _))| &adj.simplify(inp) == bank_out)
            });

            if !redundant {
                let outs = examples.iter().map(|(inp, _)| adj.simplify(inp)).collect();
                bank.push((adj, outs));
            }
            dbg!();
        }
    }
}

pub fn top_down(bank: &Bank, examples: &[(StringExpr, StringExpr)]) -> Option<StringExpr> {
    use StringExpr::*;

    let mut wl = VecDeque::new();

    // TODO: check example output, support Loc
    wl.push_back(Hole { typ: Typ::Str });

    let mut i = 0;
    while let Some(prog) = wl.pop_front() {
        dbg!();
        let prog = prog.simplify(&StringExpr::Input);
        if wl.len() > 10_000_000 {
            panic!(":(");
        }

        if i % 10_000 == 0 {
            println!("Worklist size: {}", wl.len());
        }
        i += 1;

        let mut working_copy = prog.clone();
        match working_copy.first_hole() {
            None if examples.iter().all(|(inp, out)| &prog.simplify(inp) == out) => {
                return Some(prog)
            }
            None => {
                continue;
            }
            Some(hole_ref) => {
                let fills: Vec<_> = match hole_ref {
                    Hole { typ: Typ::Str } => bank.iter().filter_map(|(expr, outs)| {
                        match outs[0] {
                            Hole {..} => unreachable!(),
                            Loc(_) | LocAdd {..} | Index {..} => None,
                            Concat {..} | Lit(_) | Slice {..} | Input => Some(expr.clone()),
                        }
                    }).collect(),
                    Hole { typ: Typ::Loc } => bank.iter().filter_map(|(expr, outs)| {
                        match outs[0] {
                            Hole {..} => unreachable!(),
                            Loc(_) | LocAdd {..} | Index {..} => Some(expr.clone()),
                            Concat {..} | Lit(_) | Slice {..} | Input => None,
                        }
                    }).collect(),
                   _ => unreachable!(),
                };

                let hole_ptr: *mut StringExpr = hole_ref;
                for fill in fills {
                    unsafe {
                        *hole_ptr = fill;
                    }

                    wl.push_back(working_copy.clone());
                }
            }
        }
    }

    None
}
