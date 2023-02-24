#![feature(io_error_other)]
#![feature(local_key_cell_methods)]
#![feature(is_some_and)]

use enumerative::top_down_vsa;

use crate::lang::StringExpr;

pub mod egg_lang;
pub mod enumerative;
pub mod lang;
pub mod vsa;

fn main() {
    // egg_lang::test();

    // let examples = vec![("First Last".to_string(), "First".to_string())];
    // let (prog, egraph) = enumerative::bottom_up_egg(&examples).unwrap();
    // let expr = prog.build_recexpr(|id| egraph.id_to_expr(id).as_ref().last().unwrap().clone());
    // dbg!(expr.to_string());

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
    // let prog = enumerative::bottom_up(&examples, 5000);
    // dbg!(prog);

    use crate::vsa::Lit;
    let res = top_down_vsa(&vec![
        (
            Lit::StringConst("I have 17 cookies".to_string()),
            Lit::StringConst("17".to_string()),
        ),
        (
            Lit::StringConst("Give me at least 3 cookies".to_string()),
            Lit::StringConst("3".to_string()),
        ),
        (
            Lit::StringConst("This number is 489".to_string()),
            Lit::StringConst("489".to_string()),
        ),
    ]);
    println!("{}, size = {}", res, res.size());
    assert_eq!(
        res.eval(&Lit::StringConst(
            "A string with the number 54234564 in the middle".to_string()
        )),
        Lit::StringConst("54234564".to_string())
    );

    let res = top_down_vsa(&vec![
        (
            Lit::StringConst("First Last".to_string()),
            Lit::StringConst("F.L.".to_string()),
        ),
        (
            Lit::StringConst("Abc Defgh".to_string()),
            Lit::StringConst("A.D.".to_string()),
        ),
        (
            Lit::StringConst("Someone Name".to_string()),
            Lit::StringConst("S.N.".to_string()),
        ),
    ]);
    println!("{}, size = {}", res, res.size());
    assert_eq!(
        res.eval(&Lit::StringConst("Another Name".to_string())),
        Lit::StringConst("A.N.".to_string())
    );
}
