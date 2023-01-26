use crate::lang::StringExpr;

use std::collections::VecDeque;

use crate::lang::*;

// this isnt really tdp i got sidetracked

// TODO: reuse bank
// should probably just follow https://dl.acm.org/doi/pdf/10.1145/3434335
// page 11
pub fn tdp(examples: &[(StringExpr, StringExpr)], max_size: usize) -> Option<StringExpr> {
    use StringExpr::*;

    if let r@Some(_) = crate::enumerative::bottom_up(examples, 300) {
        return r;
    }

    let mut wl = VecDeque::new();

    // TODO: check example output, support Loc
    wl.push_back(Hole { typ: Typ::Str });

    let mut i = 0;
    while let Some(prog) = wl.pop_front() {
        let prog = prog.simplify(&StringExpr::Input);
        if wl.len() > max_size {
            return None;
        }

        if i % 10_000 == 0 {
            println!("Worklist size: {}", wl.len());
        }
        i += 1;

        let (inp, out) = match &examples[0] {
            (StringExpr::Lit(a), StringExpr::Lit(b)) => (a, b),
            _ => panic!(),
        };
        let inp_expr = StringExpr::Lit(inp.to_string());

        let mut working_copy = prog.clone();
        match working_copy.first_hole() {
            None if examples.iter().all(|(inp, out)| &prog.simplify(inp) == out) => {
                return Some(prog)
            }
            None => {
                continue;
            }
            Some(hole_ref) => {
                let fills = match hole_ref {
                    Hole { typ: Typ::Str } => vec![
                        vec![
                            StringExpr::Input,
                            StringExpr::slice_hole(),
                            StringExpr::Lit(" ".to_string()),
                            StringExpr::concat_hole(),
                        ],
                        (1..out.len())
                            .filter_map(|i| {
                                let lhs = crate::enumerative::tdp(&[(
                                    inp_expr.clone(),
                                    StringExpr::Lit(out[0..i].to_string()),
                                )], max_size / 2);

                                let rhs = crate::enumerative::tdp(&[(
                                    inp_expr.clone(),
                                    StringExpr::Lit(out[i..].to_string()),
                                )], max_size / 2);

                                lhs.zip(rhs).map(|(lhs, rhs)| StringExpr::Concat {
                                    lhs: Box::new(lhs),
                                    rhs: Box::new(rhs),
                                })
                            })
                            .collect::<Vec<_>>(),
                        (2..out.len())
                            .filter_map(|i| {
                                inp.find(&out[0..i]).and_then(|i| {
                                    let start = crate::enumerative::bottom_up(&[
                                        (inp_expr.clone(), StringExpr::Loc(Some(i)))
                                    ], 3);
                                    let end = crate::enumerative::bottom_up(&[
                                        (inp_expr.clone(), StringExpr::Loc(Some(i + out.len())))
                                    ], 3);

                                    start.zip(end).map(|(start, end)|
                                        StringExpr::Slice {
                                            outer:Box::new(StringExpr::Input),
                                            start: Box::new(start),
                                            end: Box::new(end),
                                        }
                                    )
                                })
                            }).collect(),
                        (1..out.len())
                            .filter_map(|i| {
                                inp.find(&out[i..]).and_then(|i| {
                                    let start = crate::enumerative::bottom_up(&[
                                        (inp_expr.clone(), StringExpr::Loc(Some(i)))
                                    ], 3);
                                    let end = crate::enumerative::bottom_up(&[
                                        (inp_expr.clone(), StringExpr::Loc(Some(i + out.len())))
                                    ], 3);

                                    start.zip(end).map(|(start, end)|
                                        StringExpr::Slice {
                                            outer:Box::new(StringExpr::Input),
                                            start: Box::new(start),
                                            end: Box::new(end),
                                        }
                                    )
                                })
                            }).collect()
                            ]
                                .concat(),
                                Hole { typ: Typ::Loc } => vec![
                                    StringExpr::Loc(Some(0)),
                                    StringExpr::Loc(Some(1)),
                                    StringExpr::Loc(None),
                                    StringExpr::locadd_hole(),
                                    // StringExpr::Index { outer: Box::new(StringExpr::Input), inner: Box::new(StringExpr::string_hole()) }
                                    StringExpr::index_hole(),
                                ],
                                _ => unreachable!(),
                };

                let hole_ptr: *mut StringExpr = hole_ref;
                for fill in fills {
                    unsafe {
                        *hole_ptr = fill;
                    }

                    if working_copy.size() < 30 {
                        wl.push_back(working_copy.clone());
                    }
                }
            }
        }
    }

    None
}
