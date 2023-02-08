use std::rc::Rc;

use crate::vsa::{Fun, Lit};

type VSA = crate::vsa::VSA<Lit, Fun>;
type AST = crate::vsa::AST<Lit, Fun>;

fn top_down(examples: &[(Lit, Lit)]) -> VSA {
    let (inp, out) = &examples[0];
    learn(inp, out)
}

fn learn(inp: &Lit, out: &Lit) -> VSA {
    match out {
        Lit::StringConst(s) if s.as_str() == " " => {
            VSA::singleton(AST::Lit(Lit::StringConst(" ".to_string())))
        }
        Lit::LocConst(0) => VSA::singleton(AST::Lit(Lit::LocConst(0))),
        Lit::LocEnd => VSA::singleton(AST::Lit(Lit::LocEnd)),

        Lit::StringConst(s) => match inp {
            Lit::StringConst(inp_str) if inp_str.contains(s) => {
                let start = inp_str.find(s).unwrap();
                let end = start + s.len();
                VSA::Join {
                    op: Fun::Slice,
                    children: vec![
                        Rc::new(learn(inp, &Lit::LocConst(start))),
                        Rc::new(learn(inp, &Lit::LocConst(end))),
                    ],
                }
            }
            _ => panic!(),
        },

        Lit::LocConst(n) => match inp {
            Lit::StringConst(s) if *n == s.len() - 1 => VSA::singleton(AST::Lit(Lit::LocEnd)),
            Lit::StringConst(s) => {
                // has to be a find
                // assume lhs is always gonna be the input
                let lhs = AST::Lit(Lit::Input);
                let rhs = learn(
                    inp,
                    &Lit::StringConst(s.chars().nth(*n).unwrap().to_string()),
                );
                VSA::Join {
                    op: Fun::Find,
                    children: vec![Rc::new(VSA::singleton(lhs)), Rc::new(rhs)],
                }
            }
            _ => panic!(),
        },

        Lit::Input => panic!(),
    }
}

pub fn top_down_vsa(examples: &[(Lit, Lit)]) -> AST {
    top_down(examples).pick_one()
}

pub fn examples() -> Vec<(Lit, Lit)> {
    vec![(
        Lit::StringConst("Abc Def".to_string()),
        Lit::StringConst("Abc".to_string()),
    )]
}
