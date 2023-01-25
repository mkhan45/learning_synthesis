use std::collections::VecDeque;

use crate::lang::*;

pub fn top_down(examples: &[(StringExpr, StringExpr)]) -> Option<StringExpr> {
    use StringExpr::*;

    let mut wl = VecDeque::new();
    wl.push_back(Hole);

    while let Some(prog) = wl.pop_front() {
        if !prog.contains_hole() && examples.iter().all(|(inp, out)| &prog.simplify(inp) == out) {
            return Some(prog);
        } else if prog.contains_hole() {
            todo!()
        }
    }

    None
}
