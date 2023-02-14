use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use itertools::iproduct;

use crate::vsa::{Fun, Lit};

type VSA = crate::vsa::VSA<Lit, Fun>;
type AST = crate::vsa::AST<Lit, Fun>;

fn top_down(examples: &[(Lit, Lit)]) -> VSA {
    examples
        .into_iter()
        .map(|(inp, out)| {
            let mut bank = vec![
                AST::Lit(Lit::Input),
                AST::Lit(Lit::StringConst(" ".to_string())),
                AST::Lit(Lit::LocConst(0)),
                AST::Lit(Lit::LocConst(1)),
            ];
            let mut cache = HashMap::new();
            let mut visited = HashSet::new();

            let size = 10;
            bottom_up(inp, size, &mut cache, &mut bank);

            let res = learn(inp, out, &mut cache, &mut visited, &mut bank, 10);
            res
        })
        .inspect(|vsa| {
            println!("{:?}", vsa);
        })
        .reduce(|a, b| Rc::new(a.intersect(b.as_ref())))
        .unwrap()
        .as_ref()
        .clone()
}

// TODO:
// the cache is stupid bc it prevents generalization
// gotta bfs :/?
//
// what if the cache used observational equivalence
fn learn(
    inp: &Lit,
    out: &Lit,
    cache: &mut HashMap<Lit, Rc<VSA>>,
    visited: &mut HashSet<Lit>,
    bank: &mut Vec<AST>,
    size: usize,
) -> Rc<VSA> {
    // if size == 0 {
    //     return Rc::new(VSA::empty());
    // }

    // if let Some(res) = cache.get(out) {
    //     return res.clone();
    // }
    // if visited.contains(out) {
    //     return Rc::new(VSA::empty());
    // }
    // visited.insert(out.clone());
    // TODO: does the algorithm just not work?
    // make worklist a queue of (f, l), where l is the output to learn
    // and f(l) adds it to a VSA?
    //
    // might have to use holes like the normal top down
    //
    // TODO: I should probably unionize all of the cases
    // so that if multiple match we dont lose any options
    // probably use a multi match macro

    macro_rules! multi_match {
        ($v:expr, $($p:pat $(if $guard:expr)? => $res:expr),*) => {
            $(
                if let $p = $v {
                    if true $(&& $guard)? {
                        $res
                    }
                }
                // match $v {
                //     $p $(if $guard)? => $res,
                //     _ => {},
                // }
            )*
        };
    }

    let mut unifier = Vec::new();
    multi_match!((out, inp),
    (Lit::StringConst(s), _) if s.as_str() == " " => {
        unifier.push(VSA::singleton(AST::Lit(Lit::StringConst(" ".to_string()))))
    },
    (Lit::LocConst(0), _) => unifier.push(VSA::singleton(AST::Lit(Lit::LocConst(0)))),
    (Lit::LocConst(1), _) => unifier.push(VSA::singleton(AST::Lit(Lit::LocConst(1)))),
    (Lit::LocEnd, _) => unifier.push(VSA::singleton(AST::Lit(Lit::LocEnd))),

    (Lit::StringConst(s), Lit::StringConst(inp_str)) if inp_str.contains(s) => {
        let start = inp_str.find(s).unwrap();
        let end = start + s.len();
        let start_vsa = learn(inp, &Lit::LocConst(start), cache, visited, bank, size - 1);
        let end_vsa = learn(inp, &Lit::LocConst(end), cache, visited, bank, size - 1);
        dbg!(inp_str, s, start, end, inp_str.len(), start_vsa.pick_one(), end_vsa.pick_one());
        unifier.push(VSA::Join {
            op: Fun::Slice,
            children: vec![
                start_vsa,
                end_vsa,
            ],
        });
    },

    (Lit::StringConst(s), Lit::StringConst(_)) => {
        let set = (1..s.len())
            .map(|i| VSA::Join {
                op: Fun::Concat,
                children: vec![
                    learn(
                        inp,
                        &Lit::StringConst(s[0..i].to_string()),
                        cache,
                        visited,
                        bank,
                        size - 1,
                    ),
                    learn(
                        inp,
                        &Lit::StringConst(dbg!(s[i..].to_string())),
                        cache,
                        visited,
                        bank,
                        size - 1,
                    ),
                ],
            })
        .map(Rc::new)
            .collect();

        unifier.push(VSA::Union(set));
    },

    (Lit::LocConst(n), Lit::StringConst(s)) if *n == s.len() => {
        println!("learning LocEnd");
        unifier.push(VSA::singleton(AST::Lit(Lit::LocEnd)));
    },
    // TODO: fix bad inverse semantics for find, when there are multiple occurences
    (Lit::LocConst(n), Lit::StringConst(s)) if s.find(' ') == Some(*n) => {
        let lhs = AST::Lit(Lit::Input);
        let rhs = AST::Lit(Lit::StringConst(" ".to_string()));
        unifier.push(VSA::Join {
            op: Fun::Find,
            children: vec![Rc::new(VSA::singleton(lhs)), Rc::new(VSA::singleton(rhs))],
        });
    },
    // Lit::StringConst(s) if s.chars().nth(*n - 1).is_some_and(|x| x.is_digit(10)) => {
    //     let lhs = AST::Lit(Lit::Input);
    //     let rhs = AST::Lit(Lit::StringConst("\\d".to_string()));
    //     VSA::Join {
    //         op: Fun::Find,
    //         children: vec![Rc::new(VSA::singleton(lhs)), Rc::new(VSA::singleton(rhs))],
    //     }
    // }
    // Lit::StringConst(s) => {
    //     dbg!();
    //     // has to be a find
    //     // assume lhs is always gonna be the input
    //     let lhs = AST::Lit(Lit::Input);
    //     let rhs = learn(
    //         inp,
    //         &Lit::StringConst(s.chars().nth(*n).unwrap().to_string()),
    //         cache,
    //         visited,
    //     );
    //     VSA::Join {
    //         op: Fun::Find,
    //         children: vec![Rc::new(VSA::singleton(lhs)), rhs],
    //     }
    // }
    (Lit::LocConst(n), Lit::StringConst(s)) if *n != 0 && *n != s.len() && s.find(' ') != Some(*n) => unifier.push(VSA::Union(
            [
            Rc::new(VSA::Join {
                op: Fun::LocAdd,
                children: vec![
                    learn(inp, &Lit::LocConst(n - 1), cache, visited, bank, size - 1),
                    learn(inp, &Lit::LocConst(1), cache, visited, bank, size - 1),
                ],
            }),
            Rc::new(VSA::Join {
                op: Fun::LocSub,
                children: vec![
                    learn(inp, &Lit::LocConst(n + 1), cache, visited, bank, size - 1),
                    learn(inp, &Lit::LocConst(1), cache, visited, bank, size - 1),
                ],
            }),
            ]
            .into_iter()
            .collect(),
    ))

        );

    let res = unifier
        .into_iter()
        .map(Rc::new)
        .fold(Rc::new(VSA::empty()), |acc, x| Rc::new(VSA::unify(acc, x)));
    // let res = Rc::new(match out {
    //     Lit::StringConst(s) if s.as_str() == " " => {
    //         VSA::singleton(AST::Lit(Lit::StringConst(" ".to_string())))
    //     }
    //     Lit::LocConst(0) => VSA::singleton(AST::Lit(Lit::LocConst(0))),
    //     Lit::LocConst(1) => VSA::singleton(AST::Lit(Lit::LocConst(1))),
    //     Lit::LocEnd => VSA::singleton(AST::Lit(Lit::LocEnd)),

    //     Lit::StringConst(s) => match inp {
    //         Lit::StringConst(inp_str) if inp_str.contains(s) => {
    //             let start = inp_str.find(s).unwrap();
    //             let end = start + s.len();
    //             VSA::Join {
    //                 op: Fun::Slice,
    //                 children: vec![
    //                     learn(inp, &Lit::LocConst(start), cache, visited, bank),
    //                     learn(inp, &Lit::LocConst(end), cache, visited, bank),
    //                 ],
    //             }
    //         }
    //         Lit::StringConst(_) => {
    //             let set = (1..s.len())
    //                 .map(|i| VSA::Join {
    //                     op: Fun::Concat,
    //                     children: vec![
    //                         learn(
    //                             inp,
    //                             &Lit::StringConst(s[0..i].to_string()),
    //                             cache,
    //                             visited,
    //                             bank,
    //                         ),
    //                         learn(
    //                             inp,
    //                             &Lit::StringConst(dbg!(s[i..].to_string())),
    //                             cache,
    //                             visited,
    //                             bank,
    //                         ),
    //                     ],
    //                 })
    //             .map(Rc::new)
    //             .collect();

    //             VSA::Union(set)
    //         }
    //         _ => panic!(),
    //     },

    //     // TODO: fix bad inverse semantics for find, when there are multiple occurences
    //     Lit::LocConst(n) => match inp {
    //         Lit::StringConst(s) if *n == s.len() => VSA::singleton(AST::Lit(Lit::LocEnd)),
    //         Lit::StringConst(s) if s.find(' ') == Some(*n) => {
    //             let lhs = AST::Lit(Lit::Input);
    //             let rhs = AST::Lit(Lit::StringConst(" ".to_string()));
    //             VSA::Join {
    //                 op: Fun::Find,
    //                 children: vec![Rc::new(VSA::singleton(lhs)), Rc::new(VSA::singleton(rhs))],
    //             }
    //         }
    //         // Lit::StringConst(s) if s.chars().nth(*n - 1).is_some_and(|x| x.is_digit(10)) => {
    //         //     let lhs = AST::Lit(Lit::Input);
    //         //     let rhs = AST::Lit(Lit::StringConst("\\d".to_string()));
    //         //     VSA::Join {
    //         //         op: Fun::Find,
    //         //         children: vec![Rc::new(VSA::singleton(lhs)), Rc::new(VSA::singleton(rhs))],
    //         //     }
    //         // }
    //         // Lit::StringConst(s) => {
    //         //     dbg!();
    //         //     // has to be a find
    //         //     // assume lhs is always gonna be the input
    //         //     let lhs = AST::Lit(Lit::Input);
    //         //     let rhs = learn(
    //         //         inp,
    //         //         &Lit::StringConst(s.chars().nth(*n).unwrap().to_string()),
    //         //         cache,
    //         //         visited,
    //         //     );
    //         //     VSA::Join {
    //         //         op: Fun::Find,
    //         //         children: vec![Rc::new(VSA::singleton(lhs)), rhs],
    //         //     }
    //         // }
    //         _ => VSA::Union(
    //             [
    //             Rc::new(VSA::Join {
    //                 op: Fun::LocAdd,
    //                 children: vec![
    //                     learn(inp, &Lit::LocConst(n - 1), cache, visited, bank),
    //                     learn(inp, &Lit::LocConst(1), cache, visited, bank),
    //                 ],
    //             }),
    //             Rc::new(VSA::Join {
    //                 op: Fun::LocSub,
    //                 children: vec![
    //                     learn(inp, &Lit::LocConst(n + 1), cache, visited, bank),
    //                     learn(inp, &Lit::LocConst(1), cache, visited, bank),
    //                 ],
    //             }),
    //             ]
    //             .into_iter()
    //             .collect(),
    //         ),
    //     },

    //     Lit::Input => panic!(),
    // });

    match res.as_ref() {
        VSA::Union(s) if s.is_empty() => todo!(), //bottom up?
        _ => {}
    }

    cache.insert(out.clone(), res.clone());
    res
}

fn bottom_up(inp: &Lit, size: usize, cache: &mut HashMap<Lit, Rc<VSA>>, bank: &mut Vec<AST>) {
    // builds a VSA for a given I/O example
    // then we can add these to the cache for `learn`

    'outer: loop {
        let adjs: Vec<AST> = {
            use crate::vsa::{Fun::*, Lit::*};

            #[rustfmt::skip]
            let strings = bank.iter().filter(|e| {
                matches!(
                    e,
                    AST::Lit(Input | StringConst(_)) | AST::App { fun: Concat | Slice, .. }
                )
            });

            #[rustfmt::skip]
            let locs = bank.iter().filter(|e| {
                matches!(
                    e,
                    AST::Lit(LocConst(_) | LocEnd) | AST::App { fun: Find | LocAdd | LocSub, .. }
                )
            });

            let loc_adds = iproduct!(locs.clone(), locs.clone()).map(|(lhs, rhs)| AST::App {
                fun: Fun::LocAdd,
                args: vec![lhs.clone(), rhs.clone()],
            });

            let loc_subs = iproduct!(locs.clone(), locs.clone()).map(|(lhs, rhs)| AST::App {
                fun: Fun::LocSub,
                args: vec![lhs.clone(), rhs.clone()],
            });

            let concats = iproduct!(strings.clone(), strings.clone()).map(|(lhs, rhs)| AST::App {
                fun: Fun::Concat,
                args: vec![lhs.clone(), rhs.clone()],
            });

            let slices = iproduct!(locs.clone(), locs.clone()).map(|(start, end)| AST::App {
                fun: Fun::Slice,
                args: vec![start.clone(), end.clone()],
            });

            loc_adds.chain(loc_subs).chain(concats).chain(slices)
        }
        .collect::<Vec<_>>();

        for adj in adjs {
            if adj.size() > size {
                break 'outer;
            }

            let out = adj.eval(inp);
            if !cache.contains_key(&out) {
                cache.insert(out, Rc::new(VSA::singleton(adj.clone())));
                bank.push(adj);
            }
        }
    }
}

pub fn top_down_vsa(examples: &[(Lit, Lit)]) -> AST {
    top_down(examples).pick_one().unwrap()
}

pub fn examples() -> Vec<(Lit, Lit)> {
    vec![
        (
            Lit::StringConst("Abc Def".to_string()),
            // Lit::LocConst(3),
            Lit::StringConst("Def".to_string()),
        ),
        (
            Lit::StringConst("Hijasdf Lmnop".to_string()),
            // Lit::LocConst(7),
            Lit::StringConst("Lmnop".to_string()),
        ),
    ]
}
