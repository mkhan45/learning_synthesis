use itertools::Itertools;
use std::{collections::HashMap, collections::HashSet, fmt::Display, rc::Rc};

pub trait Language<L> {
    fn eval(&self, args: &[L], input: &L) -> L;
}

#[derive(Debug, Clone)]
pub enum VSA<L, F>
where
    L: Clone + Eq + std::hash::Hash + std::fmt::Debug + InputLit,
    F: Language<L> + std::hash::Hash + std::fmt::Debug,
{
    Leaf(HashSet<Rc<AST<L, F>>>),
    Union(Vec<Rc<VSA<L, F>>>),
    Join { op: F, children: Vec<Rc<VSA<L, F>>> },
}

impl<L, F> Default for VSA<L, F>
where
    L: std::hash::Hash + Eq + Clone + std::fmt::Debug + InputLit,
    F: Language<L> + std::hash::Hash + std::fmt::Debug,
{
    fn default() -> Self {
        VSA::Leaf(HashSet::new())
    }
}

impl<L, F> VSA<L, F>
where
    L: Clone + Eq + std::hash::Hash + std::fmt::Debug + InputLit,
    F: Language<L> + Eq + Copy + std::hash::Hash + std::fmt::Debug,
{
    pub fn empty() -> Self {
        VSA::Leaf(HashSet::new())
    }

    pub fn unify(left: Rc<VSA<L, F>>, right: Rc<VSA<L, F>>) -> Self {
        match (left.as_ref(), right.as_ref()) {
            (VSA::Leaf(l), VSA::Leaf(r)) => VSA::Leaf(l.union(r).cloned().collect()),
            (VSA::Union(u), _) => VSA::Union(
                u.iter()
                    .cloned()
                    .chain(std::iter::once(right.clone()))
                    .collect(),
            ),
            (_, VSA::Union(u)) => VSA::Union(
                u.iter()
                    .cloned()
                    .chain(std::iter::once(left.clone()))
                    .collect(),
            ),
            _ => VSA::Union(vec![left, right]),
        }
    }

    pub fn singleton(ast: AST<L, F>) -> Self {
        VSA::Leaf(std::iter::once(Rc::new(ast)).collect())
    }

    fn eval(&self, inp: &L) -> L {
        self.pick_one().unwrap().eval(inp)
        // match self {
        //     VSA::Leaf(c) => c.iter().next().unwrap().clone().eval(inp),
        //     VSA::Union(c) => c[0].eval(inp),
        //     VSA::Join { op, children } => {
        //         let cs = children
        //             .iter()
        //             .map(|vsa| vsa.clone().eval(inp))
        //             .collect::<Vec<_>>();
        //         op.eval(&cs)
        //     }
        // }
    }

    fn contains(&self, program: &AST<L, F>) -> bool {
        match self {
            VSA::Leaf(s) => s.contains(program),
            VSA::Union(vss) => vss.iter().any(|vs| vs.contains(program)),
            VSA::Join { op, children } => match program {
                AST::App { fun, args } if fun == op => args
                    .iter()
                    .all(|arg| children.iter().any(|vss| vss.contains(arg))),
                _ => false,
            },
        }
    }

    // https://dl.acm.org/doi/pdf/10.1145/2858965.2814310
    // page 10
    pub fn intersect(&self, other: &VSA<L, F>) -> VSA<L, F> {
        match (self, other) {
            (vsa, VSA::Union(union)) | (VSA::Union(union), vsa) => VSA::Union(
                union
                    .iter()
                    .map(|n1| Rc::new(n1.clone().intersect(vsa)))
                    .collect(),
            ),

            #[rustfmt::skip]
            (VSA::Join { op: l_op, .. }, VSA::Join { op: r_op, .. })
                if l_op != r_op => VSA::empty(),

            #[rustfmt::skip]
            (VSA::Join { op, children: l_children }, VSA::Join { op: _, children: r_children })
                => VSA::Join {
                    op: *op,
                    children: l_children.iter().zip(r_children).map(|(l, r)| Rc::new(l.intersect(r))).collect()
                },

            #[rustfmt::skip]
            (VSA::Join { op, children }, VSA::Leaf(s)) | (VSA::Leaf(s), VSA::Join { op, children })
                => VSA::Leaf(s.iter().filter(|pj| {
                    match pj.as_ref() {
                        AST::App { fun, args } if fun == op =>
                            args.iter().zip(children).all(|(arg, vsa)| vsa.contains(arg)),
                        _ => false
                    }
                }).cloned().collect()),

            (VSA::Leaf(l_set), VSA::Leaf(r_set)) => {
                VSA::Leaf(l_set.intersection(r_set).cloned().collect())
            }
        }
    }

    fn group_by(map: HashMap<L, Rc<VSA<L, F>>>) -> HashMap<L, Rc<VSA<L, F>>> {
        // TODO: do it in O(n)
        map.iter()
            .map(|(o1, _)| {
                (
                    o1.clone(),
                    Rc::new(VSA::Union(
                        map.iter()
                            .filter(|(o2, _)| &o1 == o2)
                            .map(|(_, v)| v.clone())
                            .collect(),
                    )),
                )
            })
            .collect()
    }

    pub fn pick_best(&self, rank: impl Fn(&AST<L, F>) -> usize + Copy) -> Option<AST<L, F>> {
        match self {
            VSA::Leaf(s) => s
                .iter()
                .sorted_by_key(|ast| rank(ast.as_ref()))
                .next()
                .map(|x| x.as_ref())
                .cloned(),
            VSA::Union(s) => s
                .iter()
                .filter_map(|vsa| vsa.pick_best(rank))
                .min_by_key(rank),
            VSA::Join { op, children } => {
                let mut args = children.iter().map(|vsa| vsa.pick_best(rank));
                if args.any(|picked| picked.is_none()) {
                    None
                } else {
                    Some(AST::App {
                        fun: *op,
                        args: children
                            .iter()
                            .map(|vsa| vsa.pick_best(rank).unwrap())
                            .collect(),
                    })
                }
            }
        }
    }

    pub fn pick_one(&self) -> Option<AST<L, F>> {
        match self {
            VSA::Leaf(s) => s.iter().next().map(|x| x.as_ref().clone()),
            VSA::Union(s) => s.iter().find_map(|vsa| vsa.pick_one()),
            VSA::Join { op, children } => {
                let mut args = children.iter().map(|vsa| vsa.pick_one());
                if args.any(|picked| picked.is_none()) {
                    None
                } else {
                    Some(AST::App {
                        fun: *op,
                        args: children.iter().map(|vsa| vsa.pick_one().unwrap()).collect(),
                    })
                }
            }
        }
    }

    fn cluster(vsa: Rc<VSA<L, F>>, input: &L) -> HashMap<L, Rc<VSA<L, F>>> {
        match vsa.as_ref() {
            VSA::Leaf(s) => VSA::group_by(
                s.iter()
                    .map(|p| {
                        (
                            p.eval(input),
                            Rc::new(VSA::Leaf(std::iter::once(p.clone()).collect())),
                        )
                    })
                    .collect(),
            ),
            VSA::Union(s) => VSA::group_by(
                // the union of all the clusters
                s.iter()
                    .map(|vsa| VSA::cluster(vsa.clone(), input))
                    .reduce(|a, b| a.into_iter().chain(b.into_iter()).collect())
                    .unwrap(),
            ),
            VSA::Join { op, children } => {
                let ns = children.iter().map(|vsa| VSA::cluster(vsa.clone(), input));
                VSA::group_by(
                    ns.map(|m| {
                        let ast = AST::App {
                            fun: *op,
                            args: m.keys().map(|l| AST::Lit(l.clone())).collect(),
                        };
                        let res = ast.eval(input);
                        (res, vsa.clone())
                    })
                    .collect(),
                )
            }
        }
    }
}

pub trait Cost {
    fn cost(&self) -> usize;
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Fun {
    Concat,
    Find,
    FindEnd,
    Slice,
    LocAdd,
    LocSub,
    Lowercase,
    Uppercase,
    ConcatMap,
    Equal,
}

impl Cost for Fun {
    fn cost(&self) -> usize {
        match self {
            Fun::Concat => 2,
            _ => 1,
        }
    }
}

pub trait InputLit {
    fn is_input(&self) -> bool;
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum Lit {
    StringConst(String),
    LocConst(usize),
    BoolConst(bool),
    LocEnd,
    Input,
}

impl InputLit for Lit {
    fn is_input(&self) -> bool {
        self == &Lit::Input
    }
}

impl Cost for Lit {
    fn cost(&self) -> usize {
        match self {
            Lit::Input | Lit::LocEnd => 0,
            _ => 1,
        }
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub enum AST<L, F>
where
    L: std::hash::Hash + std::fmt::Debug + InputLit,
    F: Language<L> + std::hash::Hash + std::fmt::Debug,
{
    App { fun: F, args: Vec<AST<L, F>> },
    Lit(L),
}

impl Language<Lit> for Fun {
    fn eval(&self, args: &[Lit], input: &Lit) -> Lit {
        match self {
            Fun::Concat => match args {
                [Lit::StringConst(lhs), Lit::StringConst(rhs)] => {
                    Lit::StringConst(format!("{}{}", lhs, rhs))
                }
                _ => panic!(),
            },
            Fun::ConcatMap => {
                // TODO: can't do this yet because of how eval works
                todo!()
                // let mut buf = String::new();
                // Lit::StringConst(buf)
            }
            Fun::Find => match args {
                [Lit::StringConst(outer), Lit::StringConst(inner), index] => {
                    let i = match index {
                        Lit::LocConst(i) => *i,
                        Lit::LocEnd => outer.len(),
                        _ => panic!(),
                    };

                    use crate::enumerative::regex;
                    let re = regex(inner);
                    let mut found = re.find_iter(outer).map(|m| Lit::LocConst(m.start()));

                    found.nth(i).unwrap_or(Lit::LocEnd)
                }
                _ => panic!(),
            },
            Fun::FindEnd => match args {
                [Lit::StringConst(outer), Lit::StringConst(inner), index] => {
                    let i = match index {
                        Lit::LocConst(i) => *i,
                        Lit::LocEnd => outer.len(),
                        _ => panic!(),
                    };

                    use crate::enumerative::regex;
                    let re = regex(inner);
                    let mut found = re.find_iter(outer).map(|m| Lit::LocConst(m.end()));

                    found.nth(i).unwrap_or(Lit::LocEnd)
                }
                _ => panic!(),
            },
            Fun::Slice => match (args, input) {
                ([Lit::LocConst(start), Lit::LocConst(end)], Lit::StringConst(s))
                    if start <= end && end <= &s.len() =>
                {
                    Lit::StringConst(s[*start..*end].to_owned())
                }
                ([Lit::LocConst(start), Lit::LocEnd], Lit::StringConst(s)) if *start <= s.len() => {
                    Lit::StringConst(s[*start..].to_owned())
                }
                _ => Lit::StringConst("".to_string()),
            },
            Fun::LocAdd => match args {
                [Lit::LocConst(a), Lit::LocConst(b)] => Lit::LocConst(a + b),
                [Lit::LocEnd, _] | [_, Lit::LocEnd] => Lit::LocEnd,
                _ => panic!(),
            },
            Fun::LocSub => match args {
                [Lit::LocConst(a), Lit::LocConst(b)] => {
                    Lit::LocConst(a.checked_sub(*b).unwrap_or(0))
                }
                [Lit::LocEnd, _] | [_, Lit::LocEnd] => Lit::LocEnd,
                _ => panic!(),
            },
            Fun::Equal => match (args, input) {
                ([Lit::LocConst(a), Lit::LocConst(b)], _) => Lit::BoolConst(a == b),
                // ([Lit::LocEnd, Lit::LocEnd], _) => Lit::BoolConst(true),
                ([Lit::LocConst(a), Lit::LocEnd] | [Lit::LocEnd, Lit::LocConst(a)], Lit::StringConst(s)) => {
                    Lit::BoolConst(*a == s.len())
                }
                ([Lit::StringConst(a), Lit::StringConst(b)], _) => Lit::BoolConst(a == b),
                ([Lit::Input, Lit::StringConst(b)] | [Lit::StringConst(b), Lit::Input], Lit::StringConst(s)) => {
                    Lit::BoolConst(b == s)
                }
                _ => Lit::BoolConst(false),
            },
            Fun::Lowercase => match args {
                [Lit::StringConst(s)] => Lit::StringConst(s.to_lowercase()),
                _ => panic!(),
            },
            Fun::Uppercase => match args {
                [Lit::StringConst(s)] => Lit::StringConst(s.to_uppercase()),
                _ => panic!(),
            },
        }
    }
}

impl<L, F> AST<L, F>
where
    L: Clone + std::hash::Hash + std::fmt::Debug + InputLit,
    F: Language<L> + Copy + std::hash::Hash + std::fmt::Debug,
{
    pub fn eval(&self, inp: &L) -> L {
        match self {
            AST::Lit(l) if l.is_input() => inp.clone(),
            AST::Lit(l) => l.clone(),
            AST::App { fun, args } => {
                let evaled = args.iter().map(|ast| ast.eval(inp)).collect::<Vec<_>>();
                fun.eval(&evaled, inp)
            }
        }
    }

    pub fn size(&self) -> usize {
        match self {
            AST::Lit(_) => 1,
            AST::App { args, .. } => 1 + args.iter().map(AST::size).sum::<usize>(),
        }
    }
}

impl<L, F> Cost for AST<L, F>
where
    L: Clone + std::hash::Hash + std::fmt::Debug + InputLit + Cost,
    F: Language<L> + Copy + std::hash::Hash + std::fmt::Debug + Cost,
{
    fn cost(&self) -> usize {
        match self {
            AST::Lit(l) => l.cost(),
            AST::App { fun, args } => fun.cost() + args.iter().map(AST::size).sum::<usize>(),
        }
    }
}

impl Display for AST<Lit, Fun> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // stupid rewrite because of dumb witness
            AST::App {
                fun: Fun::Concat,
                args,
            } if args[0] == AST::Lit(Lit::StringConst("".to_string())) => {
                // assume it's only 2 args bc it shouldnt be variadic anyway
                let (_fst, snd) = (args[0].clone(), args[1].clone());
                write!(f, "{snd}")
            }
            AST::App {
                fun: Fun::Concat,
                args,
            } if args[1] == AST::Lit(Lit::StringConst("".to_string())) => {
                // assume it's only 2 args bc it shouldnt be variadic anyway
                let (fst, _snd) = (args[0].clone(), args[1].clone());
                write!(f, "{fst}")
            }
            AST::App {
                fun: Fun::Concat,
                args,
            } => {
                // assume it's only 2 args bc it shouldnt be variadic anyway
                let (fst, snd) = (args[0].clone(), args[1].clone());
                write!(f, "({fst} <> {snd})")
            }
            AST::App {
                fun: Fun::ConcatMap,
                args,
            } => {
                // assume it's only 2 args bc it shouldnt be variadic anyway
                let (split, subprog) = (args[0].clone(), args[1].clone());
                write!(f, "X.split({split}).concat_map(λX.{subprog})")
            }
            AST::App {
                fun: Fun::Find,
                args,
            } => {
                let (fst, snd, i) = (args[0].clone(), args[1].clone(), args[2].clone());
                write!(f, "{fst}.find({snd}, {i})")
            }
            AST::App {
                fun: Fun::FindEnd,
                args,
            } => {
                let (fst, snd, i) = (args[0].clone(), args[1].clone(), args[2].clone());
                write!(f, "{fst}.find_end({snd}, {i})")
            }
            AST::App {
                fun: Fun::Slice,
                args,
            } => {
                // assume only input is sliced
                let (fst, snd) = (args[0].clone(), args[1].clone());
                write!(f, "(X[{fst}..{snd}])")
            }
            AST::App {
                fun: Fun::LocAdd,
                args,
            } => {
                let a = args[0].clone();
                let b = args[1].clone();
                write!(f, "({a} + {b})")
            }
            AST::App {
                fun: Fun::LocSub,
                args,
            } => {
                let a = args[0].clone();
                let b = args[1].clone();
                write!(f, "({a} - {b})")
            }
            AST::App {
                fun: Fun::Lowercase,
                args,
            } => {
                let x = args[0].clone();
                write!(f, "{x}.lower()")
            }
            AST::App {
                fun: Fun::Uppercase,
                args,
            } => {
                let x = args[0].clone();
                write!(f, "{x}.upper()")
            }
            AST::App {
                fun: Fun::Equal,
                args,
            } => {
                let a = args[0].clone();
                let b = args[1].clone();
                write!(f, "({a} == {b})")
            }
            AST::Lit(Lit::StringConst(s)) => write!(f, "'{}'", s),
            AST::Lit(Lit::LocConst(n)) => write!(f, "{}", n),
            AST::Lit(Lit::BoolConst(b)) => write!(f, "{}", b),
            AST::Lit(Lit::LocEnd) => write!(f, "$"),
            AST::Lit(Lit::Input) => write!(f, "X"),
        }
    }
}

trait DefaultASTDisplay {}

impl<L, F> Display for AST<L, F>
where
    L: Clone + std::hash::Hash + std::fmt::Debug + InputLit,
    F: Language<L> + Copy + std::hash::Hash + std::fmt::Debug + DefaultASTDisplay,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AST::App { fun, args } => {
                let args = args
                    .iter()
                    .fold(String::new(), |acc, arg| format!("{}{} ", acc, arg));
                write!(f, "({:?} [ {}])", fun, args)
            }
            AST::Lit(l) => write!(f, "{:?}", l),
        }
    }
}

impl std::fmt::Display for Lit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Lit::StringConst(s) => {
                f.write_str("\"")?; 
                f.write_str(s)?;
                f.write_str("\"")?; 
                Ok(())
            }
            Lit::LocConst(l) => f.write_str(&l.to_string()),
            Lit::BoolConst(b) => f.write_str(&b.to_string()),
            Lit::LocEnd => f.write_str("$"),
            Lit::Input => f.write_str("X"),
        }
    }
}
