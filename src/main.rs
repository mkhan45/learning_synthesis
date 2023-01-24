use crate::lang::StringExpr;

pub mod enumerative;
pub mod lang;

fn main() {
    let examples = vec![
        (
            StringExpr::Lit("First Last".to_owned()),
            StringExpr::Lit("F L".to_owned()),
        ),
        (
            StringExpr::Lit("Abc Def".to_owned()),
            StringExpr::Lit("A D".to_owned()),
        ),
    ];
    let prog = enumerative::bottom_up(&examples);
    dbg!(prog);
}
