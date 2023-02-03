use std::{collections::HashSet, marker::PhantomData, rc::Rc};

pub trait Language<L> {
    fn eval(&self, args: &[L]) -> L;
}

pub enum VSA<L: Clone + Eq + std::hash::Hash, F: Language<L> + std::hash::Hash> {
    Leaf(HashSet<Rc<AST<L, F>>>),
    Union(Vec<Rc<VSA<L, F>>>),
    Join { op: F, children: Vec<Rc<VSA<L, F>>> },
}

impl<L: Clone + Eq + std::hash::Hash, F: Language<L> + Eq + Copy + std::hash::Hash> VSA<L, F> {
    fn eval(&self) -> L {
        match self {
            VSA::Leaf(c) => c.iter().next().unwrap().clone().eval(),
            VSA::Union(c) => c[0].eval(),
            VSA::Join { op, children } => {
                let cs = children
                    .iter()
                    .map(|vsa| vsa.clone().eval())
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

    fn ranking(&self, cmp: impl Fn(&VSA<L, F>) -> usize) -> VSA<L, F> {
        todo!()
    }

    fn cluster(&self, examples: &Vec<(L, L)>) -> Vec<(L, VSA<L, F>)> {
        todo!()
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum Fun {
    Concat,
    Find,
    Slice,
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum Lit {
    StringConst(String),
    LocConst(usize),
    LocEnd,
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct AST<L: std::hash::Hash, F: Language<L> + std::hash::Hash> {
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

impl<L: Clone + std::hash::Hash, F: Language<L> + Copy + std::hash::Hash> AST<L, F> {
    pub fn eval(&self) -> L {
        let evaled = self.args.iter().map(AST::eval).collect::<Vec<_>>();
        match self.fun {
            Some(fun) => fun.eval(&evaled),
            None => evaled[0].clone(),
        }
    }
}
