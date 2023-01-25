pub struct Hole;

pub enum LitPrim {
    Comma,
    Space,
}

#[derive(Clone, PartialEq, Eq)]
pub enum StringExpr {
    Loc(Option<usize>),
    LocAdd {
        lhs: Box<StringExpr>,
        rhs: Box<StringExpr>,
    },
    Lit(String),
    Concat {
        lhs: Box<StringExpr>,
        rhs: Box<StringExpr>,
    },
    Index {
        outer: Box<StringExpr>,
        inner: Box<StringExpr>,
    },
    Slice {
        outer: Box<StringExpr>,
        start: Box<StringExpr>,
        end: Box<StringExpr>,
    },
    Input,
    Hole,
}

impl std::fmt::Debug for StringExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Loc(i) => write!(f, "{}", i.map(|x| x as isize).unwrap_or(-1)),
            Self::LocAdd { lhs, rhs } => write!(f, "({:?} + {:?})", lhs, rhs),
            Self::Lit(l) => write!(f, "'{}'", l),
            Self::Concat { lhs, rhs } => write!(f, "({:?} <> {:?})", lhs, rhs),
            Self::Index { outer, inner } => write!(f, "({:?}.find({:?}))", outer, inner),
            Self::Slice { outer, start, end } => write!(f, "({:?}[{:?}..{:?}])", outer, start, end),
            Self::Input => write!(f, "X"),
            Self::Hole => write!(f, "_"),
        }
    }
}

impl StringExpr {
    pub fn simplify(&self, input: &StringExpr) -> StringExpr {
        use StringExpr::*;

        match self {
            Lit(_) | Loc(_) | Hole => self.clone(),
            LocAdd { lhs, rhs } => match (lhs.as_ref(), rhs.as_ref()) {
                (Loc(Some(lhs)), Loc(Some(rhs))) => Loc(Some(lhs + rhs)),
                _ => Loc(None),
            },
            Input => input.clone(),
            Concat { lhs, rhs } => match (lhs.simplify(input), rhs.simplify(input)) {
                (Lit(lhs), Lit(rhs)) => Lit(format!("{}{}", lhs, rhs)),
                (nlhs, nrhs) => Concat {
                    lhs: Box::new(nlhs),
                    rhs: Box::new(nrhs),
                },
            },
            Index { outer, inner } => match (outer.simplify(input), inner.simplify(input)) {
                (Lit(lhs), Lit(rhs)) => Loc(lhs.find(&rhs)),
                (outer, inner) => Index {
                    outer: Box::new(outer),
                    inner: Box::new(inner),
                },
            },
            Slice { outer, start, end } => {
                match (
                    outer.simplify(input),
                    start.simplify(input),
                    end.simplify(input),
                ) {
                    (Lit(lhs), Loc(Some(start)), Loc(None)) if start < lhs.len() => {
                        Lit(lhs[start..].to_string())
                    }
                    (Lit(lhs), Loc(Some(start)), Loc(Some(end)))
                        if start < lhs.len() && end < lhs.len() && start < end =>
                    {
                        Lit(lhs[start..end].to_string())
                    }
                    (Lit(_), Loc(_), Loc(_)) => Lit("".to_string()),
                    (outer, start, end) => Slice {
                        outer: Box::new(outer),
                        start: Box::new(start),
                        end: Box::new(end),
                    },
                }
            }
        }
    }

    pub fn contains_hole(&self) -> bool {
        use StringExpr::*;

        match self {
            Hole => true,
            Loc(_) | Lit(_) | Input => false,
            LocAdd { lhs, rhs } => lhs.contains_hole() || rhs.contains_hole(),
            Concat { lhs, rhs } => lhs.contains_hole() || rhs.contains_hole(),
            Index { outer, inner } => outer.contains_hole() || inner.contains_hole(),
            Slice { outer, start, end } => outer.contains_hole() || start.contains_hole() || end.contains_hole(),
        }
    }
    // pub fn replace_hole(self, goal: &StringExpr) -> Option<StringExpr> {
    //     use StringExpr::*;

    //     match (self.simplify(), goal) {
    //         (Hole, _) => Some(goal.clone()),
    //         (Lit(lhs), Lit(rhs)) if &lhs == rhs => Some(goal.clone()),
    //         _ => todo!(),
    //     }
    // }
}
