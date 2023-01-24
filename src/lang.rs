pub struct Hole;

pub enum LitPrim {
    Comma,
    Space,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StringExpr {
    Loc(usize),
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

impl StringExpr {
    pub fn simplify(self, input: &StringExpr) -> StringExpr {
        use StringExpr::*;

        match self {
            Lit(_) | Loc(_) | Hole => self,
            Input => input.clone(),
            Concat { lhs, rhs } => match (lhs.simplify(input), rhs.simplify(input)) {
                (Lit(lhs), Lit(rhs)) => Lit(format!("{}{}", lhs, rhs)),
                (nlhs, nrhs) => Concat {
                    lhs: Box::new(nlhs),
                    rhs: Box::new(nrhs),
                },
            },
            Index { outer, inner } => match (outer.simplify(input), inner.simplify(input)) {
                (Lit(lhs), Lit(rhs)) => Loc(lhs.find(&rhs).unwrap_or(999)),
                (outer, inner) => Index {
                    outer: Box::new(outer),
                    inner: Box::new(inner),
                },
            },
            Slice { outer, start, end } => {
                match (outer.simplify(input), start.simplify(input), end.simplify(input)) {
                    (Lit(lhs), Loc(start), Loc(end)) if start < lhs.len() && end < lhs.len() => Lit(lhs[start..end].to_string()),
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
