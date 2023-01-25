use std::collections::VecDeque;

use crate::lang::*;

pub fn top_down(examples: &[(StringExpr, StringExpr)]) -> Option<StringExpr> {
    use StringExpr::*;

    let mut wl = VecDeque::new();

    // TODO: check example output, support Loc
    wl.push_back(Hole { typ: Typ::Str });

    let mut i = 0;
    while let Some(prog) = wl.pop_front() {
        if wl.len() > 10_000_000 {
            panic!(":(");
        }

        if i % 10_000 == 0 {
            println!("Worklist size: {}", wl.len());
        }
        i += 1;
    
        let mut working_copy = prog.clone();
        match working_copy.first_hole() {
            None if examples.iter().all(|(inp, out)| &prog.simplify(inp) == out) => {
                return Some(prog)
            }
            None => {
                continue;
            },
            Some(hole_ref) => {
                let fills = match hole_ref {
                    Hole { typ: Typ::Str } => vec![
                        StringExpr::Input, 
                        StringExpr::concat_hole(),
                        StringExpr::slice_hole(),
                        StringExpr::Lit(" ".to_string()),
                    ],
                    Hole { typ: Typ::Loc } => vec![
                        StringExpr::Loc(Some(0)),
                        StringExpr::Loc(Some(1)),
                        StringExpr::Loc(None),
                        // StringExpr::Index { outer: Box::new(StringExpr::Input), inner: Box::new(StringExpr::string_hole()) }
                        StringExpr::index_hole(),
                    ],
                    _ => unreachable!(),
                };

                let hole_ptr: *mut StringExpr = hole_ref;
                for fill in fills {
                    unsafe {
                        *hole_ptr = fill;
                    }

                    if working_copy.size() < 30 {
                        wl.push_back(working_copy.clone());
                    }
                }
            }
        }
    }

    None
}
