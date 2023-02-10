use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::vsa::{Fun, Lit};

type VSA = crate::vsa::VSA<Lit, Fun>;
type AST = crate::vsa::AST<Lit, Fun>;

fn top_down(examples: &[(Lit, Lit)]) -> VSA {
    let mut bank = Vec::new();

    examples
        .into_iter()
        .map(|(inp, out)| learn(inp, out, &mut HashMap::new(), &mut HashSet::new(), &mut bank))
        .reduce(|a, b| Rc::new(a.intersect(b.as_ref())))
        .unwrap()
        .as_ref()
        .clone()
}

fn learn(
    inp: &Lit,
    out: &Lit,
    cache: &mut HashMap<Lit, Rc<VSA>>,
    visited: &mut HashSet<Lit>,
    bank: &mut Vec<AST>,
) -> Rc<VSA> {
    if let Some(res) = cache.get(out) {
        return res.clone();
    }
    if visited.contains(out) {
        return Rc::new(VSA::empty());
    }
    visited.insert(out.clone());
    // TODO: does the algorithm just not work?
    // make worklist a queue of (f, l), where l is the output to learn
    // and f(l) adds it to a VSA?
    //
    // might have to use holes like the normal top down
    //
    // TODO: I should probably unionize all of the cases
    // so that if multiple match we dont lose any options
    let res = Rc::new(match out {
        Lit::StringConst(s) if s.as_str() == " " => {
            VSA::singleton(AST::Lit(Lit::StringConst(" ".to_string())))
        }
        Lit::LocConst(0) => VSA::singleton(AST::Lit(Lit::LocConst(0))),
        Lit::LocConst(1) => VSA::singleton(AST::Lit(Lit::LocConst(1))),
        Lit::LocEnd => VSA::singleton(AST::Lit(Lit::LocEnd)),

        Lit::StringConst(s) => match inp {
            Lit::StringConst(inp_str) if inp_str.contains(s) => {
                let start = inp_str.find(s).unwrap();
                let end = start + s.len();
                VSA::Join {
                    op: Fun::Slice,
                    children: vec![
                        learn(inp, &Lit::LocConst(start), cache, visited, bank),
                        learn(inp, &Lit::LocConst(end), cache, visited, bank),
                    ],
                }
            }
            Lit::StringConst(_) => {
                let set = (1..s.len() - 1)
                    .map(|i| VSA::Join {
                        op: Fun::Concat,
                        children: vec![
                            Rc::new(VSA::Join {
                                op: Fun::Slice,
                                children: vec![
                                    Rc::new(VSA::singleton(AST::Lit(Lit::LocConst(0)))),
                                    Rc::new(VSA::singleton(AST::Lit(Lit::LocConst(i)))),
                                ],
                            }),
                            learn(
                                inp,
                                &Lit::StringConst(s[i + 1..].to_string()),
                                cache,
                                visited,
                                bank,
                            ),
                        ],
                    })
                    .map(Rc::new)
                    .collect();

                VSA::Union(set)
            }
            _ => panic!(),
        },

        // TODO: fix bad inverse semantics for find, when there are multiple occurences
        Lit::LocConst(n) => match inp {
            Lit::StringConst(s) if *n == s.len() => VSA::singleton(AST::Lit(Lit::LocEnd)),
            Lit::StringConst(s) if s.chars().nth(*n - 1).unwrap_or('.') == ' ' => {
                let lhs = AST::Lit(Lit::Input);
                let rhs = AST::Lit(Lit::StringConst(" ".to_string()));
                VSA::Join {
                    op: Fun::Find,
                    children: vec![Rc::new(VSA::singleton(lhs)), Rc::new(VSA::singleton(rhs))],
                }
            }
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
            _ => VSA::Union(
                [
                    Rc::new(VSA::Join {
                        op: Fun::LocAdd,
                        children: vec![
                            learn(inp, &Lit::LocConst(n - 1), cache, visited, bank),
                            learn(inp, &Lit::LocConst(1), cache, visited, bank),
                        ],
                    }),
                    Rc::new(VSA::Join {
                        op: Fun::LocSub,
                        children: vec![
                            learn(inp, &Lit::LocConst(n + 1), cache, visited, bank),
                            learn(inp, &Lit::LocConst(1), cache, visited, bank),
                        ],
                    }),
                ]
                .into_iter()
                .collect(),
            ),
        },

        Lit::Input => panic!(),
    });

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

    loop {
        let adjs: Vec<AST> = Vec::new(); // TODO

        for adj in adjs {
            if adj.size() > size {
                continue;
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
            Lit::StringConst("AIX 5.1".to_string()),
            Lit::StringConst("5.1".to_string()),
        ),
        (
            Lit::StringConst("Linux Linux 2.6 Linux".to_string()),
            Lit::StringConst("2.6".to_string()),
        ),
    ]
}