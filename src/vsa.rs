use std::rc::Rc;

pub trait Language<L> {
    fn eval(&self, args: &[L]) -> L;
}

pub enum VSA<L: Clone, F: Language<L>> {
    Leaf(Vec<L>),
    Union(Vec<Rc<VSA<L, F>>>),
    Join { 
        op: F,
        children: Vec<Rc<VSA<L, F>>>,
    }
}

impl<L: Clone, F: Language<L>> VSA<L, F> {
    fn eval(&self) -> L {
        match self {
            VSA::Leaf(c) => c[0].clone(),
            VSA::Union(c) => c[0].eval(),
            VSA::Join { op, children } => {
                let cs = children.iter().map(|vsa| vsa.clone().eval()).collect::<Vec<_>>();
                op.eval(&cs)
            }
        }
    }

    fn intersect(&self, other: &VSA<L, F>) -> VSA<L, F> {
        todo!()
    }

    fn ranking(&self, cmp: impl Fn(&VSA<L, F>) -> usize) -> VSA<L, F> {
        todo!()
    }

    fn cluster(&self, examples: &Vec<(L, L)>) -> Vec<(L, VSA<L, F>)> {
        todo!()
    }
}

pub enum Fun {
    Concat,
    Find,
    Slice,
}

pub enum Lit {
    StringConst(String),
    LocConst(usize),
    LocEnd,
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
            Fun::Find => {
                match args {
                    [Lit::StringConst(outer), Lit::StringConst(inner)] => {
                        outer.find(inner).map(|l| Lit::LocConst(l)).unwrap_or(Lit::LocEnd)
                    },
                    _ => panic!(),
                }
            },
            Fun::Slice => {
                match args {
                    [Lit::StringConst(s), Lit::LocConst(start), Lit::LocConst(end)] => {
                        Lit::StringConst(s[*start..*end].to_owned())
                    },
                    [Lit::StringConst(s), Lit::LocConst(start), Lit::LocEnd] => {
                        Lit::StringConst(s[*start..].to_owned())
                    },
                    [Lit::StringConst(_), Lit::LocEnd, _] => {
                        Lit::StringConst("".to_owned())
                    },
                    _ => panic!(),
                }
            }
        }
    }
}
