use crate::lang::StringExpr;

pub mod enumerative;
pub mod lang;

fn main() {
    let examples = vec![
        (StringExpr::Lit("First Last".to_owned()), StringExpr::Lit("First".to_owned())),
    ];
    let prog = enumerative::search::synthesize(&examples);
    dbg!(prog);
}
