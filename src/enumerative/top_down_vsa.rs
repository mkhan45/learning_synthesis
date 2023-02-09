use std::rc::Rc;

use crate::vsa::{Fun, Lit};

type VSA = crate::vsa::VSA<Lit, Fun>;
type AST = crate::vsa::AST<Lit, Fun>;

fn top_down(examples: &[(Lit, Lit)]) -> VSA {
    // i dont want to think about loops right now
    let (inp, out) = &examples[0];
    let vsa1 = learn(inp, out);

    let (inp2, out2) = &examples[1];
    let vsa2 = learn(inp2, out2);

    println!("VSA 1: {}", vsa1.pick_one());
    println!("VSA 2: {}", vsa2.pick_one());
    vsa1.intersect(&vsa2)
}

fn learn(inp: &Lit, out: &Lit) -> VSA {
    dbg!();
    // TODO: does the algorithm just not work?
    // make worklist a queue of (f, l), where l is the output to learn
    // and f(l) adds it to a VSA?
    //
    // might have to use holes like the normal top down
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
            Lit::StringConst(inp_str) => {
                let set = (1..inp_str.len() - 1)
                    .map(|i| VSA::Join {
                        op: Fun::Concat,
                        children: vec![
                            Rc::new(VSA::Join {
                                op: Fun::Slice,
                                children: vec![
                                    Rc::new(VSA::singleton(AST::Lit(Lit::LocConst(0)))),
                                    Rc::new(VSA::singleton(AST::Lit(Lit::LocConst(i)))),
                                ],
                            }),
                            Rc::new(learn(inp, &Lit::StringConst(inp_str[i + 1..].to_string()))),
                        ],
                    })
                    .map(Rc::new)
                    .collect();

                VSA::Union(set)
            }
            _ => panic!(),
        },

        Lit::LocConst(n) => match inp {
            Lit::StringConst(s) if *n == s.len() => VSA::singleton(AST::Lit(Lit::LocEnd)),
            Lit::StringConst(s) if s.chars().nth(*n - 1).unwrap_or('.') == ' ' => {
                let lhs = AST::Lit(Lit::Input);
                let rhs = AST::Lit(Lit::StringConst(" ".to_string()));
                VSA::Join {
                    op: Fun::Find,
                    children: vec![Rc::new(VSA::singleton(lhs)), Rc::new(VSA::singleton(rhs))],
                }
            }
            // Lit::StringConst(s) => {
            //     dbg!();
            //     // has to be a find
            //     // assume lhs is always gonna be the input
            //     let lhs = AST::Lit(Lit::Input);
            //     let rhs = learn(
            //         inp,
            //         &Lit::StringConst(s.chars().nth(*n).unwrap().to_string()),
            //     );
            //     VSA::Join {
            //         op: Fun::Find,
            //         children: vec![Rc::new(VSA::singleton(lhs)), Rc::new(rhs)],
            //     }
            // }
            _ => VSA::Union([
                Rc::new(VSA::Join { op: Fun::LocInc, children: vec![Rc::new(learn(inp, &Lit::LocConst(n - 1)))] }),
            ].into_iter().collect())
        },

        Lit::Input => panic!(),
    }
}

pub fn top_down_vsa(examples: &[(Lit, Lit)]) -> AST {
    top_down(examples).pick_one()
}

pub fn examples() -> Vec<(Lit, Lit)> {
    vec![
        (
            Lit::StringConst("Abc Def".to_string()),
            Lit::StringConst("Abc ".to_string()),
        ),
        (
            Lit::StringConst("QWErty Uiop".to_string()),
            Lit::StringConst("QWErty ".to_string()),
        )
    ]
}
