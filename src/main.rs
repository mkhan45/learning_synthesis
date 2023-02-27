#![feature(io_error_other)]
#![feature(local_key_cell_methods)]
#![feature(is_some_and)]

use enumerative::top_down_vsa;

use crate::lang::StringExpr;

pub mod bank;
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

macro_rules! test_duet_str {
    ($name:ident, $($inp:expr => $out:expr),+; $($test_inp:expr => $test_out:expr),+) => {
        #[test]
        fn $name() {
            use crate::vsa::Lit;
            let res = top_down_vsa(&vec![
                $(
                    (
                        Lit::StringConst($inp.to_string()),
                        Lit::StringConst($out.to_string()),
                    ),
                )+
            ]);
            println!("{}, size = {}", res, res.size());

            $(
                assert_eq!(
                    res.eval(&Lit::StringConst($test_inp.to_string())),
                    Lit::StringConst($test_out.to_string())
                );
            )+
        }
    };
}

// Run these with cargo test --release -- --nocapture to see the output

test_duet_str!(
    test_duet_date,
    "01/15/2013" => "01/2013",
    "03/07/2011" => "03/2011",
    "05/09/2009" => "05/2009";

    "01/02/03" => "01/03",
    "09/02/07" => "09/07"
);

test_duet_str!(
    test_duet_numbers,
    "I have 17 cookies" => "17",
    "Give me at least 3 cookies" => "3",
    "This number is 489" => "489";

    "A string with the number 54234564 in the middle" => "54234564",
    "36" => "36",
    "Number at the end 74" => "74"
);

test_duet_str!(
    test_duet_abbrev,
    "First Last" => "F.L.",
    "Abc Defgh" => "A.D.",
    "Someone Name" => "S.N.";

    "Another Name" => "A.N."
);

test_duet_str!(
    test_duet_model_no,
    "Tire Pressure ABC123873 Monitor" => "ABC123873",
    " Air conditioner GHF211 maintenance" => "GHF211";

    " Oil Life ABC849999999021 gauge" => "ABC849999999021"
);

// fails :(
test_duet_str!(
    test_duet_url,
    "http://www.example.com" => "example",
    "https://www.apple.com/uk/mac" => "apple";

    "https://www.google.com" => "google"
);
