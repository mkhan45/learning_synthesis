use std::{collections::HashMap, collections::HashSet, marker::PhantomData, rc::Rc};

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
            VSA::Join { op, children } => match program.fun {
                Some(f) if f == *op => program
                    .args
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
                    match pj.fun {
                        Some(f) if f == *op => 
                            pj.args.iter().all(|arg| children.iter().any(|cs| cs.clone().contains(arg))),
                        _ => false,
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

    fn cluster(&self, input: &L) -> HashMap<L, Rc<VSA<L, F>>> {
        match self {
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
                    .map(|vsa| vsa.cluster(input))
                    .reduce(|a, b| a.into_iter().chain(b.into_iter()).collect())
                    .unwrap(),
            ),
            VSA::Join { op, children } => {
                let ns = children.iter().map(|vsa| vsa.cluster(input));
                todo!()
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
pub struct AST<L, F>
where
    L: std::hash::Hash + std::fmt::Debug,
    F: Language<L> + std::hash::Hash + std::fmt::Debug,
{
    pub fun: Option<F>,
    pub args: Vec<AST<L, F>>,
    phantom_l: PhantomData<L>,
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
        let evaled = self
            .args
            .iter()
            .map(|ast| ast.eval(inp))
            .collect::<Vec<_>>();
        match self.fun {
            Some(fun) => fun.eval(&evaled),
            None => evaled[0].clone(),
        }
    }
}
