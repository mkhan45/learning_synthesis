use std::{
    collections::{HashMap, HashSet},
    num::NonZeroUsize,
    rc::Rc,
    sync::RwLock,
};

use itertools::iproduct;
use lru::LruCache;
use regex::Regex;

use crate::{
    bank::Bank,
    vsa::{Fun, Lit, Cost},
};

use lazy_static::lazy_static;

type VSA = crate::vsa::VSA<Lit, Fun>;
type AST = crate::vsa::AST<Lit, Fun>;

macro_rules! loc_pat {
    () => { AST::Lit(Lit::LocConst(_) | Lit::LocEnd) | AST::App { fun: Fun::Find | Fun::LocAdd | Fun::LocSub, .. } };
}

lazy_static! {
    // TODO: figure out ideal cache size
    pub static ref CACHE: RwLock<LruCache<String, Regex>> = RwLock::new(LruCache::new(NonZeroUsize::new(2000).unwrap()));
    pub static ref EMPTY_REGEX: Regex = Regex::new(".").unwrap();
}

pub fn regex(s: &String) -> Regex {
    let mut cache_writer = CACHE.write().unwrap();
    if cache_writer.contains(s) {
        cache_writer.get(s).unwrap().clone()
    } else {
        // cache_writer.push(s.clone(), Regex::new(s).unwrap_or(regex(&".".to_string())));
        cache_writer.push(s.clone(), Regex::new(s).unwrap_or(EMPTY_REGEX.clone()));
        cache_writer.get(s).unwrap().clone()
    }
}

// TODO:
// add a substitute function

pub fn top_down(examples: &[(Lit, Lit)]) -> Option<AST> {
    let mut bank = Bank::new();
    let mut regex_bank = Bank::new();
    let mut all_cache = HashMap::new();

    let mut char_sets = examples.iter().map(|(inp, out)| match (inp, out) {
        (Lit::StringConst(inp), Lit::StringConst(out)) => inp
            .chars()
            .chain(out.chars())
            .filter(|c| !c.is_alphanumeric())
            .map(|c| match c {
                '.' => Lit::StringConst("\\.".to_string()),
                '{' => Lit::StringConst("\\{".to_string()),
                '}' => Lit::StringConst("\\{".to_string()),
                _ => Lit::StringConst(c.to_string()),
            })
            .collect::<HashSet<_>>(),
        _ => HashSet::new(),
    });
    let intersection = char_sets
        .next()
        .map(|s1| {
            s1.iter()
                .filter(|c| char_sets.clone().all(|s2| s2.contains(c)))
                .cloned()
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    // dbg!(&intersection);

    // TODO:
    // a cache that is only applied to regexes
    for prim in [
        Lit::Input,
        Lit::StringConst("".to_string()),
        Lit::StringConst(" ".to_string()),
        Lit::StringConst(".".to_string()),
        Lit::LocConst(0),
        Lit::LocConst(1),
        Lit::LocEnd,
    ]
    .into_iter()
    .chain(intersection.clone().into_iter())
    {
        bank.size_mut(1).push(AST::Lit(prim.clone()));
        all_cache.insert(
            std::iter::repeat(prim.clone())
                .take(examples.len())
                .collect(),
            Rc::new(VSA::singleton(AST::Lit(prim.clone()))),
        );
    }

    for prim in [
        Lit::StringConst("\\d".to_string()),
        Lit::StringConst("\\b".to_string()),
        Lit::StringConst("[a-z]".to_string()),
        Lit::StringConst("[A-Z]".to_string()),
    ]
    .into_iter()
    .chain(intersection.into_iter())
    {
        regex_bank.size_mut(1).push(AST::Lit(prim.clone()));
    }

    let enable_bools = examples.iter().any(|(_, out)| matches!(out, Lit::BoolConst(_)));

    let mut size = 1;
    let inps = examples.iter().map(|(inp, _)| inp);

    while size <= 6 {
        bottom_up(
            inps.clone(),
            size,
            &mut all_cache,
            &mut bank,
            &mut regex_bank,
            enable_bools
        );
        // dbg!(bank.total_entries());
        let mut ex_vsas = examples.iter().enumerate().map(|(i, (inp, out))| {
            let mut cache: HashMap<Lit, Rc<VSA>> = HashMap::new();
            for (outs, vsa) in all_cache.iter() {
                if let Some(v) = cache.get_mut(&outs[i]) {
                    *v = Rc::new(VSA::unify(vsa.clone(), v.clone()));
                } else {
                    cache.insert(outs[i].clone(), vsa.clone());
                }
            }

            learn(inp, out, &mut cache, &bank)
        });

        let mut res = ex_vsas.next().unwrap();

        // TODO:
        // instead of pick_best, pick the best 10, and then
        // check if it works on all examples
        for vsa in ex_vsas {
            if let Some(prog) = res.pick_best(|ast| ast.cost()) {
                if examples.iter().all(|(inp, out)| prog.eval(inp) == *out) {
                    break;
                };
            }

            res = Rc::new(res.intersect(vsa.as_ref()));
        }

        match res.pick_best(|ast| ast.cost()) {
            res @ Some(_) => return res,
            None => size += 1,
        }
    }

    None
}

// TODO:
// there's still an issue with cycles here
// maybe still needs a queue
fn learn(inp: &Lit, out: &Lit, cache: &mut HashMap<Lit, Rc<VSA>>, bank: &Bank<AST>) -> Rc<VSA> {
    let mut unifier = Vec::new();
    if let Some(res) = cache.get(out) {
        unifier.push(res.as_ref().clone());
        // return res.clone();
    }

    macro_rules! universal_witness {
        ($p:pat) => {
            bank.entries.iter().flat_map(|entry| entry.iter().filter(|ast| matches!(ast, $p)))
        }
    }

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

    multi_match!((out, inp),
    // (Lit::StringConst(s), _) if s.as_str() == " " => {
    //     unifier.push(VSA::singleton(AST::Lit(Lit::StringConst(" ".to_string()))))
    // },
    // (Lit::StringConst(s), _) if s.as_str() == "." => {
    //     unifier.push(VSA::singleton(AST::Lit(Lit::StringConst(".".to_string()))))
    // },
    //
    // TODO:
    // this makes it impossible to learn in one shot
    (Lit::StringConst(s), _) => {
        unifier.push(VSA::singleton(AST::Lit(Lit::StringConst(s.clone()))))
    },

    (Lit::BoolConst(b), _) => {
        unifier.push(VSA::singleton(AST::Lit(Lit::BoolConst(*b))))
    },

    (Lit::LocConst(n), _) => {
        unifier.push(VSA::singleton(AST::Lit(Lit::LocConst(*n))))
    },

    (Lit::LocConst(n), Lit::StringConst(inp_str)) if inp_str.len() == *n => {
        unifier.push(VSA::singleton(AST::Lit(Lit::LocEnd)));
    },

    (Lit::BoolConst(b), _) => {
        let s = iproduct!(universal_witness!(loc_pat!()), universal_witness!(loc_pat!())).map(|(lhs, rhs)| {
            AST::App {
                fun: Fun::Equal,
                args: vec![lhs.clone(), rhs.clone()],
            }
        }).map(Rc::new).collect();
        unifier.push(VSA::Leaf(s));
    },

    (Lit::StringConst(s), Lit::StringConst(inp_str)) if s.contains(inp_str) => {
        let re = regex(inp_str);

        re.find_iter(s)
            .map(|m| {
                let start = m.start();
                let end = m.end();
                let start_vsa = learn(inp, &Lit::StringConst(s[0..start].to_string()), cache, bank);
                let end_vsa = learn(inp, &Lit::StringConst(s[end..].to_string()), cache, bank);
                // dbg!(start, end, s[0..start].to_string(), s[end..].to_string(), start_vsa.clone(), end_vsa.clone());
                // TODO: maybe add a simplify function to the AST
                VSA::Join {
                    op: Fun::Concat,
                    children: vec![
                        start_vsa,
                        Rc::new(VSA::Join {
                            op: Fun::Concat,
                            children: vec![
                                learn(inp, &Lit::Input, cache, bank),
                                end_vsa,
                            ],
                        }),
                    ],
                }
            })
        .for_each(|vsa| unifier.push(vsa));
        },

        (Lit::StringConst(s), Lit::StringConst(inp_str)) if inp_str.contains(s) => {
            let re = regex(s);

            re.find_iter(inp_str)
                .map(|m| {
                    let start = m.start();
                    let end = m.end();
                    let start_vsa = learn(inp, &Lit::LocConst(start), cache, bank);
                    let end_vsa = learn(inp, &Lit::LocConst(end), cache, bank);
                    VSA::Join {
                        op: Fun::Slice,
                        children: vec![
                            start_vsa,
                            end_vsa,
                        ],
                    }
                })
            .for_each(|vsa| unifier.push(vsa));
            // let start = inp_str.find(s).unwrap();
            // let end = start + s.len();
            // // dbg!(s, start, end);
            // let start_vsa = learn(inp, &Lit::LocConst(start), cache);
            // let end_vsa = learn(inp, &Lit::LocConst(end), cache);
            // unifier.push(VSA::Join {
            //     op: Fun::Slice,
            //     children: vec![
            //         start_vsa,
            //         end_vsa,
            //     ],
            // });
            },

            (Lit::StringConst(s), Lit::StringConst(inp_str)) if !inp_str.contains(s) && !s.contains(inp_str) => {
                let set = (1..s.len())
                    .map(|i| VSA::Join {
                        op: Fun::Concat,
                        children: vec![
                            learn(
                                inp,
                                &Lit::StringConst(s[0..i].to_string()),
                                cache,
                                bank,
                            ),
                            learn(
                                inp,
                                &Lit::StringConst(s[i..].to_string()),
                                cache,
                                bank,
                            ),
                        ],
                    })
                .map(Rc::new)
                .collect();

                unifier.push(VSA::Union(set));
            }

    // TODO: figure out the index
    // (Lit::LocConst(n), Lit::StringConst(s)) if s.chars().nth(*n).is_some_and(|ch| ch == ' ') => {
    //     let lhs = Rc::new(VSA::singleton(AST::Lit(Lit::Input)));
    //     let space = cache.get(&Lit::StringConst(" ".to_string())).unwrap().clone();
    //     let wb = cache.get(&Lit::StringConst("\\b".to_string())).unwrap().clone();

    //     unifier.push(VSA::Join {
    //         op: Fun::Find,
    //         children: vec![lhs.clone(), space],
    //     });

    //     if s.chars().nth(n - 1).is_some_and(|ch| ch.is_alphanumeric()) {
    //         unifier.push(VSA::Join {
    //             op: Fun::Find,
    //             children: vec![lhs, wb],
    //         });
    //     }
    // }
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
    bank: &mut Bank<AST>,
    regex_bank: &mut Bank<AST>,
    enable_bools: bool,
) {
    // dbg!(size);
    bank.grow_to(size);
    regex_bank.grow_to(size);
    // builds a VSA for a given I/O example
    // then we can add these to the cache for `learn`

    // TODO: a better way to keep track of size, make the bank store
    // by size so that we can just directly make expressions of the correct size
    //
    // TODO: probably remove LocAdd and LocSub in favor for LocInc and LocDec or something
    use crate::vsa::{Fun::*, Lit::*};

    #[rustfmt::skip]
    let regexes_of_size = |n: usize| {
        regex_bank.size(n).iter()
    };

    #[rustfmt::skip]
    let strings_of_size = |n: usize| {
        bank.size(n).iter().filter(|e| {
            matches!(
                e,
                AST::Lit(Input | StringConst(_)) | AST::App { fun: Concat | Slice, .. }
            )
        })
    };

    #[rustfmt::skip]
    let locs_of_size = |n: usize| {
        bank.size(n).iter().filter(|e| {
            matches!(
                e,
                AST::Lit(LocConst(_) | LocEnd) | AST::App { fun: Find | LocAdd | LocSub, .. }
            )
        })
    };

    #[rustfmt::skip]
    let bools_of_size = |n: usize| {
        bank.size(n).iter().filter(|e| {
            matches!(
                e,
                AST::Lit(BoolConst(_)) | AST::App { fun: Equal, .. }
            )
        })
    };

    let adjs: Vec<AST> = {
        let loc_adds = (1..size).flat_map(|i| {
            let lhs_size = i;
            let rhs_size = size - i;
            // dbg!(locs_of_size(dbg!(lhs_size)).collect::<Vec<_>>());
            iproduct!(locs_of_size(lhs_size), locs_of_size(rhs_size)).map(|(lhs, rhs)| AST::App {
                fun: Fun::LocAdd,
                args: vec![lhs.clone(), rhs.clone()],
            })
        });

        let loc_subs = (1..size).flat_map(|i| {
            let lhs_size = i;
            let rhs_size = size - i;
            iproduct!(locs_of_size(lhs_size), locs_of_size(rhs_size)).map(|(lhs, rhs)| AST::App {
                fun: Fun::LocSub,
                args: vec![lhs.clone(), rhs.clone()],
            })
        });

        // I guess the concat witness function is complete
        // so this isn't needed
        let concats = (1..size).flat_map(|i| {
            let lhs_size = i;
            let rhs_size = size - i;
            iproduct!(strings_of_size(lhs_size), strings_of_size(rhs_size)).map(|(lhs, rhs)| {
                AST::App {
                    fun: Fun::Concat,
                    args: vec![lhs.clone(), rhs.clone()],
                }
            })
        });

        let re_concats = (1..size).flat_map(|i| {
            let lhs_size = i;
            let rhs_size = size - i;
            iproduct!(regexes_of_size(lhs_size), regexes_of_size(rhs_size)).map(|(lhs, rhs)| {
                AST::App {
                    fun: Fun::Concat,
                    args: vec![lhs.clone(), rhs.clone()],
                }
            })
        });

        let finds = (1..size - 1).flat_map(|l| {
            (l + 1..size).flat_map(move |r| {
                let lhs_size = l;
                let rhs_size = r - l;
                let index_size = size - r;
                // dbg!(lhs_size, rhs_size, index_size);
                iproduct!(
                    strings_of_size(lhs_size),
                    // strings_of_size(rhs_size),
                    strings_of_size(rhs_size).chain(regexes_of_size(rhs_size)),
                    locs_of_size(index_size)
                )
                .flat_map(|(lhs, rhs, index)| {
                    [
                        AST::App {
                            fun: Fun::Find,
                            args: vec![lhs.clone(), rhs.clone(), index.clone()],
                        },
                        AST::App {
                            fun: Fun::FindEnd,
                            args: vec![lhs.clone(), rhs.clone(), index.clone()],
                        },
                    ]
                })
            })
        });

        let slices = (1..size).flat_map(|i| {
            let lhs_size = i;
            let rhs_size = size - i;
            iproduct!(locs_of_size(lhs_size), locs_of_size(rhs_size)).map(|(lhs, rhs)| AST::App {
                fun: Fun::Slice,
                args: vec![lhs.clone(), rhs.clone()],
            })
        });

        let re_groups = (1..size - 1).flat_map(|size| {
            strings_of_size(size).map(|e| AST::App {
                fun: Fun::Concat,
                args: vec![e.clone(), AST::Lit(Lit::StringConst("+".to_string()))],
            })
        });
        // dbg!(re_groups.clone().collect::<Vec<_>>());

        let loc_eq_size = if enable_bools { size } else { 0 };
        let loc_eqs = (1..loc_eq_size).flat_map(|i| {
            let lhs_size = i;
            let rhs_size = size - i;
            iproduct!(locs_of_size(lhs_size), locs_of_size(rhs_size)).map(|(lhs, rhs)| AST::App {
                fun: Fun::Equal,
                args: vec![lhs.clone(), rhs.clone()],
            })
        });

        // loc_adds
        //     .chain(loc_subs)
        re_concats
            // .chain(concats)
            .chain(slices)
            .chain(finds)
            .chain(re_groups)
            .chain(loc_eqs)
    }
    .filter(|adj| {
        let outs = inps.clone().map(|inp| adj.eval(inp)).collect::<Vec<_>>();
        use std::collections::hash_map::Entry;

        // dbg!(adj.size(), size);
        // dbg!(adj.size(), size, bank.len());
        if let Entry::Vacant(e) = cache.entry(outs) {
            e.insert(Rc::new(VSA::singleton(adj.clone())));
            true
        } else {
            false
        }
    })
    .collect::<Vec<_>>();

    bank.size_mut(size).extend(adjs);
    // dbg!(&bank);
}

pub fn top_down_vsa(examples: &[(Lit, Lit)]) -> AST {
    top_down(examples).unwrap()
}
