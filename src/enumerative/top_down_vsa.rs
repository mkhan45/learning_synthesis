use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use itertools::iproduct;

use crate::vsa::{Fun, Lit};

type VSA = crate::vsa::VSA<Lit, Fun>;
type AST = crate::vsa::AST<Lit, Fun>;

fn top_down(examples: &[(Lit, Lit)]) -> VSA {
    let mut bank = Vec::new();
    let mut all_cache = HashMap::new();
    for prim in [
        Lit::Input,
        Lit::StringConst(" ".to_string()),
        Lit::StringConst(".".to_string()),
        Lit::LocConst(0),
        Lit::LocConst(1),
    ] {
        bank.push(AST::Lit(prim.clone()));
        all_cache.insert(
            std::iter::repeat(prim.clone())
                .take(examples.len())
                .collect(),
            Rc::new(VSA::singleton(AST::Lit(prim.clone()))),
        );
    }

    let mut size = 3;
    let inps = examples.iter().map(|(inp, _)| inp);
    bottom_up(inps.clone(), size, &mut all_cache, &mut bank);

    while size < 5 {
        let res = examples
            .into_iter()
            .enumerate()
            .map(|(i, (inp, out))| {
                // let mut cache: HashMap<Lit, Rc<VSA>> = all_cache.iter().map(|(outs, vsa)| {
                //     (outs[i].clone(), vsa.clone())
                // }).collect();
                let mut cache: HashMap<Lit, Rc<VSA>> = HashMap::new();
                for (outs, vsa) in all_cache.iter() {
                    if let Some(v) = cache.get_mut(&outs[i]) {
                        *v = Rc::new(VSA::unify(vsa.clone(), v.clone()));
                    } else {
                        cache.insert(outs[i].clone(), vsa.clone());
                    }
                }
                let mut visited = HashSet::new();

                // TODO TODO: preprocess cache to map output -> union of all VSAs p s.t. p(inp) = out for
                // the specific inp
                //
                // Might be better to have bottom up only work for one input at a time? Probably not
                //
                // Remove visited, LocAdd, LocSub -> wonder how rewrite rules would work
                //
                // The other TODOs are probably dumb
                let res = learn(inp, out, &mut cache, &mut visited, &mut bank, 10);
                res
            })
            // .inspect(|vsa| {
            //     println!("VSA: {:?}", vsa);
            // })
            .reduce(|a, b| Rc::new(a.intersect(b.as_ref())))
            .unwrap()
            .as_ref()
            .clone();

        if res.pick_one().is_some() {
            return res;
        } else {
            size += 1;
            bottom_up(inps.clone(), size, &mut all_cache, &mut bank);
        }
    }

    VSA::empty()
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
    if size == 0 {
        return Rc::new(VSA::empty());
    }

    if let Some(res) = cache.get(out) {
        return res.clone();
    }
    if visited.contains(out) {
        return Rc::new(VSA::empty());
    }
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
                match $v {
                    $p $(if $guard)? => $res,
                    _ => {},
                }
            )*
        };
    }

    let mut unifier = Vec::new();
    multi_match!((out, inp),
    (Lit::StringConst(s), _) if s.as_str() == " " => {
        unifier.push(VSA::singleton(AST::Lit(Lit::StringConst(" ".to_string()))))
    },
    (Lit::StringConst(s), _) if s.as_str() == "." => {
        unifier.push(VSA::singleton(AST::Lit(Lit::StringConst(".".to_string()))))
    },
    (Lit::LocConst(0), _) => unifier.push(VSA::singleton(AST::Lit(Lit::LocConst(0)))),
    (Lit::LocConst(1), _) => unifier.push(VSA::singleton(AST::Lit(Lit::LocConst(1)))),
    (Lit::LocEnd, _) => unifier.push(VSA::singleton(AST::Lit(Lit::LocEnd))),

    (Lit::StringConst(s), Lit::StringConst(inp_str)) if inp_str.contains(s) => {
        let start = inp_str.find(s).unwrap();
        let end = start + s.len();
        let start_vsa = learn(inp, &Lit::LocConst(start), cache, visited, bank, size - 1);
        let end_vsa = learn(inp, &Lit::LocConst(end), cache, visited, bank, size - 1);
        // dbg!(inp_str, s, start, end, inp_str.len(), start_vsa.pick_one(), end_vsa.pick_one());
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
                        &Lit::StringConst(s[i..].to_string()),
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
        // println!("learning LocEnd");
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
    // (Lit::LocConst(n), Lit::StringConst(_)) if *n > 0 => unifier.push(VSA::Union(
    //         [
    //         Rc::new(VSA::Join {
    //             op: Fun::LocAdd,
    //             children: vec![
    //                 learn(inp, &Lit::LocConst(n - 1), cache, visited, bank, size - 1),
    //                 learn(inp, &Lit::LocConst(1), cache, visited, bank, size - 1),
    //             ],
    //         }),
    //         Rc::new(VSA::Join {
    //             op: Fun::LocSub,
    //             children: vec![
    //                 learn(inp, &Lit::LocConst(n + 1), cache, visited, bank, size - 1),
    //                 learn(inp, &Lit::LocConst(1), cache, visited, bank, size - 1),
    //             ],
    //         }),
    //         ]
    //         .into_iter()
    //         .collect(),
    // ))

    );

    let res = unifier
        .into_iter()
        .map(Rc::new)
        .fold(Rc::new(VSA::empty()), |acc, x| Rc::new(VSA::unify(acc, x)));

    match res.as_ref() {
        VSA::Union(s) if s.is_empty() => todo!(), //bottom up?
        _ => {}
    }

    // cache.insert(out.clone(), res.clone());
    res
}

fn bottom_up<'a>(
    inps: impl Iterator<Item = &'a Lit> + Clone,
    size: usize,
    cache: &mut HashMap<Vec<Lit>, Rc<VSA>>,
    bank: &mut Vec<AST>,
) {
    dbg!(size);
    // builds a VSA for a given I/O example
    // then we can add these to the cache for `learn`

    // TODO: a better way to keep track of size, make the bank store
    // by size so that we can just directly make expressions of the correct size
    //
    // TODO: probably remove LocAdd and LocSub in favor for LocInc and LocDec or something
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
            // dbg!(locs.clone().collect::<Vec<_>>());

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

            let finds = iproduct!(strings.clone(), strings.clone()).map(|(lhs, rhs)| AST::App {
                fun: Fun::Find,
                args: vec![lhs.clone(), rhs.clone()],
            });

            let slices = iproduct!(locs.clone(), locs.clone()).map(|(start, end)| AST::App {
                fun: Fun::Slice,
                args: vec![start.clone(), end.clone()],
            });

            // loc_adds
            //     .chain(loc_subs)
            //     .chain(concats)
            //     .chain(slices)
            //     .chain(finds)
            concats.chain(slices).chain(finds)
        }
        .collect::<Vec<_>>();

        let mut some_small = false;
        for adj in adjs {
            let outs = inps.clone().map(|inp| adj.eval(inp)).collect::<Vec<_>>();

            if !cache.contains_key(&outs) {
                if adj.size() <= size {
                    // println!("{}", adj);
                    some_small = true;
                }

                cache.insert(outs, Rc::new(VSA::singleton(adj.clone())));
                bank.push(adj);
            }
        }

        if !some_small {
            break 'outer;
        }
    }

    dbg!(size);
}

pub fn top_down_vsa(examples: &[(Lit, Lit)]) -> AST {
    top_down(examples).pick_one().unwrap()
}

pub fn examples() -> Vec<(Lit, Lit)> {
    vec![
        (
            Lit::StringConst("Abc Def".to_string()),
            // Lit::LocConst(3),
            Lit::StringConst("A.D.".to_string()),
        ),
        (
            Lit::StringConst("Hijasdf Lmnop".to_string()),
            // Lit::LocConst(7),
            Lit::StringConst("H.L.".to_string()),
        ),
    ]
}
