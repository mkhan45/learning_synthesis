use std::{collections::HashMap, collections::HashSet, rc::Rc, fmt::Display};

pub trait Language<L> {
    fn eval(&self, args: &[L]) -> L;
}

#[derive(Debug)]
pub enum VSA<L, F>
where
    L: Clone + Eq + std::hash::Hash + std::fmt::Debug,
    F: Language<L> + std::hash::Hash + std::fmt::Debug,
{
    Leaf(HashSet<Rc<AST<L, F>>>),
    Union(Vec<Rc<VSA<L, F>>>),
    Join { op: F, children: Vec<Rc<VSA<L, F>>> },
}

impl<L, F> Default for VSA<L, F>
where
    L: std::hash::Hash + Eq + Clone + std::fmt::Debug,
    F: Language<L> + std::hash::Hash + std::fmt::Debug,
{
    fn default() -> Self {
        VSA::Leaf(HashSet::new())
    }
}

impl<L, F> VSA<L, F>
where
    L: Clone + Eq + std::hash::Hash + std::fmt::Debug,
    F: Language<L> + Eq + Copy + std::hash::Hash + std::fmt::Debug,
{
    pub fn empty() -> Self {
        VSA::Leaf(HashSet::new())
    }

    pub fn singleton(ast: AST<L, F>) -> Self {
        VSA::Leaf(std::iter::once(Rc::new(ast)).collect())
    }

    fn eval(&self, inp: &L) -> L {
        match self {
            VSA::Leaf(c) => c.iter().next().unwrap().clone().eval(inp),
            VSA::Union(c) => c[0].eval(inp),
            VSA::Join { op, children } => {
                let cs = children
                    .iter()
                    .map(|vsa| vsa.clone().eval(inp))
                    .collect::<Vec<_>>();
                op.eval(&cs)
            }
        }
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
    fn intersect(&self, other: &VSA<L, F>) -> VSA<L, F> {
        match (self, other) {
            (vsa, VSA::Union(union)) | (VSA::Union(union), vsa) => VSA::Union(
                union
                    .iter()
                    .map(|n1| Rc::new(n1.clone().intersect(vsa)))
                    .collect(),
            ),

            #[rustfmt::skip]
            (VSA::Join { op: l_op, .. }, VSA::Join { op: r_op, .. })
                if l_op != r_op => VSA::Leaf(HashSet::with_capacity(0)),

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
                            args.iter().all(|arg| children.iter().any(|cs| cs.clone().contains(arg))),
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

    fn ranking(&self, cmp: impl Fn(&VSA<L, F>) -> usize) -> VSA<L, F> {
        todo!()
    }

    pub fn pick_one(&self) -> AST<L, F> {
        match self {
            VSA::Leaf(s) => s.iter().next().unwrap().as_ref().clone(),
            VSA::Union(s) => s[0].pick_one(),
            VSA::Join { op, children } => AST::App {
                fun: *op,
                args: children.iter().map(|vsa| vsa.pick_one()).collect(),
            },
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

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Fun {
    Concat,
    Find,
    Slice,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum Lit {
    StringConst(String),
    LocConst(usize),
    LocEnd,
    Input,
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub enum AST<L, F>
where
    L: std::hash::Hash + std::fmt::Debug,
    F: Language<L> + std::hash::Hash + std::fmt::Debug,
{
    App { fun: F, args: Vec<AST<L, F>> },
    Lit(L),
}

impl Language<Lit> for Fun {
    fn eval(&self, args: &[Lit]) -> Lit {
        match self {
            Fun::Concat => {
                let mut buf = String::new();
                for arg in args {
                    match arg {
                        Lit::StringConst(s) => buf.push_str(s),
                        _ => panic!(),
                    }
                }
                Lit::StringConst(buf)
            }
            Fun::Find => match args {
                [Lit::StringConst(outer), Lit::StringConst(inner)] => outer
                    .find(inner)
                    .map(|l| Lit::LocConst(l))
                    .unwrap_or(Lit::LocEnd),
                _ => panic!(),
            },
            Fun::Slice => match args {
                [Lit::StringConst(s), Lit::LocConst(start), Lit::LocConst(end)] => {
                    Lit::StringConst(s[*start..*end].to_owned())
                }
                [Lit::StringConst(s), Lit::LocConst(start), Lit::LocEnd] => {
                    Lit::StringConst(s[*start..].to_owned())
                }
                [Lit::StringConst(_), Lit::LocEnd, _] => Lit::StringConst("".to_owned()),
                _ => panic!(),
            },
        }
    }
}

impl<L, F> AST<L, F>
where
    L: Clone + std::hash::Hash + std::fmt::Debug,
    F: Language<L> + Copy + std::hash::Hash + std::fmt::Debug,
{
    pub fn eval(&self, inp: &L) -> L {
        match self {
            AST::Lit(l) => l.clone(),
            AST::App { fun, args } => {
                let evaled = args.iter().map(|ast| ast.eval(inp)).collect::<Vec<_>>();
                fun.eval(&evaled)
            }
        }
    }
}

impl<L, F> Display for AST<L, F>
where
    L: Clone + std::hash::Hash + std::fmt::Debug,
    F: Language<L> + Copy + std::hash::Hash + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AST::App { fun, args } => {
                let args = args.iter().fold(String::new(), |acc, arg| format!("{}{} ", acc, arg));
                write!(f, "({:?} [ {}])", fun, args)
            }
            AST::Lit(l) => write!(f, "{:?}", l),
        }
    }
}
