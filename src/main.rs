#![feature(io_error_other)]

use egg::Language;

use crate::lang::StringExpr;

pub mod egg_lang;
pub mod enumerative;
pub mod lang;

fn main() {
    // egg_lang::test();

    let examples = vec![("First Last".to_string(), "First".to_string())];
    let (prog, egraph) = enumerative::bottom_up_egg(&examples).unwrap();
    let expr = prog.build_recexpr(|id| egraph.id_to_expr(id).as_ref().last().unwrap().clone());
    dbg!(expr.to_string());

    // let examples = vec![
    //     (
    //         StringExpr::Lit("First Last".to_owned()),
    //         StringExpr::Lit("F L".to_owned()),
    //     ),
    //     (
    //         StringExpr::Lit("Abc Def".to_owned()),
    //         StringExpr::Lit("A D".to_owned()),
    //     ),
    // ];
    // let prog = enumerative::bottom_up(&examples);
    // dbg!(prog);
}
