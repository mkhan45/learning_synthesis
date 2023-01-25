use itertools::iproduct;

use crate::lang::*;

pub fn bottom_up(examples: &[(StringExpr, StringExpr)], max_size: usize) -> Option<StringExpr> {
    use StringExpr::*;

    let mut bank = Vec::new();
    let mut add_to_bank = |prog: StringExpr| {
        let simpl = prog.simplify(&StringExpr::Input);
        let outs = examples
            .iter()
            .map(|(inp, _)| simpl.simplify(inp))
            .collect::<Vec<_>>();
        bank.push((simpl, outs));
    };
    add_to_bank(Loc(Some(0)));
    add_to_bank(Loc(Some(1)));
    add_to_bank(Loc(None));
    add_to_bank(Lit(" ".to_string()));
    add_to_bank(Input);

    for _ in 0..10 {
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
        .map(|prog| prog.simplify(&StringExpr::Input))
        .collect::<Vec<_>>();

        for adj in adjs {
            if adj.size() > max_size {
                continue;
            }

            if examples.iter().all(|(inp, out)| &adj.simplify(inp) == out) {
                return Some(adj);
            } else {
                let redundant = bank.iter().any(|(_, bank_outs)| {
                    bank_outs
                        .iter()
                        .zip(examples.iter())
                        .all(|(bank_out, (inp, _))| &adj.simplify(inp) == bank_out)
                });

                // overfitted prune
                let too_many_spaces = examples.iter().all(|(inp, out)| {
                    if let (StringExpr::Lit(inp), StringExpr::Lit(out), StringExpr::Lit(prog_out)) =
                        (inp, out, adj.simplify(inp))
                    {
                        let count_spaces = |x: &String| x.chars().filter(|ch| *ch == ' ').count();
                        let inp_spaces = count_spaces(inp);
                        let out_spaces = count_spaces(out);
                        let prog_spaces = count_spaces(&prog_out);

                        (out_spaces <= inp_spaces) && (prog_spaces > inp_spaces)
                    } else {
                        false
                    }
                });

                if !redundant && !too_many_spaces {
                    if bank.len() % 100 == 0 {
                        println!("Bank Size: {}", bank.len());
                    }
                    if bank.len() > max_size {
                        return None;
                    }


                    let outs = examples.iter().map(|(inp, _)| adj.simplify(inp)).collect();
                    bank.push((adj, outs));
                }
            }
        }
    }

    None
}
