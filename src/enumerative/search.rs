use itertools::iproduct;

use crate::lang::*;

pub fn bottom_up(examples: &[(StringExpr, StringExpr)]) -> Option<StringExpr> {
    use StringExpr::*;

    let mut bank = vec![
        Loc(Some(0)),
        Loc(Some(1)),
        Loc(None),
        Lit(" ".to_string()),
        Input,
    ];

    for _ in 0..10 {
        let adjs = {
            let strings = bank
                .iter()
                .filter(|e| matches!(e, Lit(_) | Concat { .. } | Slice { .. } | Input));
            let locs = bank.iter().filter(|e| matches!(e, Loc(_) | Index { .. }));

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
        .filter(|new_prog| {
            !iproduct!(examples.iter(), bank.iter())
                .any(|((inp, _), bank_prog)| new_prog.simplify(inp) == bank_prog.simplify(inp))
            // if new_prog == (&StringExpr::LocAdd {
            //     lhs: Box::new(StringExpr::Loc(Some(1))),
            //     rhs: Box::new(StringExpr::Loc(Some(1))),
            // }) {
            //     dbg!(res, new_prog.simplify(&StringExpr::Input));
            // };
            // res
        })
        .map(|prog| prog.simplify(&StringExpr::Input))
        .collect::<Vec<_>>();

        for adj in adjs {
            if examples.iter().all(|(inp, out)| &adj.simplify(inp) == out) {
                return Some(adj);
            } else {
                let redundant = iproduct!(examples.iter(), bank.iter())
                    .any(|((inp, _), bank_prog)| adj.simplify(inp) == bank_prog.simplify(inp));

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
                    bank.push(adj);
                }
            }
        }
    }

    None
}
