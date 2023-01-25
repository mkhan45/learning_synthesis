use egg::{define_language, rewrite as rw, Id, Language, RecExpr, Rewrite, Symbol};

define_language! {
    pub enum StringExprEgg {
        Loc(usize),
        Lit(String),
        "$" = End,
        "X" = Input,
        "+" = Add([Id; 2]),
        "<>" = Concat([Id; 2]),
        "[]" = Slice([Id; 3]),
        "find" = Find([Id; 2]),
        Symbol(Symbol),
    }
}

pub fn simplify(expr: RecExpr<StringExprEgg>, input: &String) -> StringExprEgg {
    let nodes = expr.as_ref();

    fn eval(id: Id, nodes: &[StringExprEgg], input: &String) -> StringExprEgg {
        use egg::Symbol;
        use StringExprEgg::{Symbol as Sym, *};

        let e = |id| eval(id, nodes, input);

        match &nodes[usize::from(id)] {
            Input => Lit(input.clone()),
            n @ (Loc(_) | Lit(_) | Symbol(_) | End) => n.clone(),

            Add([lhs, rhs]) => match (e(*lhs), e(*rhs)) {
                (Loc(lhs), Loc(rhs)) => Loc(lhs + rhs),
                _ => Symbol(Symbol::new("Error")),
            },

            Concat([lhs, rhs]) => match (e(*lhs), e(*rhs)) {
                (Lit(lhs), Lit(rhs)) => Lit(format!("{}{}", lhs, rhs)),
                _ => Symbol(Symbol::new("Error")),
            },

            Slice([lhs, start, end]) => match (e(*lhs), e(*start), e(*end)) {
                (Lit(lhs), Loc(start), End) => Lit(lhs[start..].to_string()),
                (Lit(lhs), Loc(start), Loc(end)) if start <= end && end <= lhs.len() => {
                    Lit(lhs[start..end].to_string())
                }
                _ => Symbol(Symbol::new("Error")),
            },

            Find([outer, inner]) => match (e(*outer), e(*inner)) {
                (Lit(lhs), Lit(rhs)) => match lhs.find(&rhs) {
                    Some(i) => Loc(i),
                    None => Sym(Symbol::new("Error")),
                },
                _ => Symbol(Symbol::new("Error")),
            },
        }
    }

    eval((nodes.len() - 1).into(), nodes, input)
}

pub fn test() {
    let ast: RecExpr<StringExprEgg> = "(+ 3 5)".parse().unwrap();
    dbg!(&ast);
    dbg!(simplify(ast, &"".to_string()));

    // let ast: RecExpr<StringExprEgg> = "'asdf'".parse().unwrap();
}
