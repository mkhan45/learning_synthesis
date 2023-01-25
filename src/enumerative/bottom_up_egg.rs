use crate::egg_lang::{self, simplify, StringExprEgg as StringExpr};
use egg::*;
use itertools::{iproduct, Itertools};

pub struct ObservationalEquivalence<'a> {
    examples: &'a [(String, String)],
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Typ {
    Str,
    Loc,
    Symbol,
}

// idk why but it tries to unify things when it shouldn't
// I'll try to understand e-graphs better before continuing
//
// I also haven't found any writing about using e-graphs for
// synthesis directly in this way, so maybe I'm misunderstanding something
impl<'a> Analysis<StringExpr> for ObservationalEquivalence<'a> {
    type Data = (Vec<StringExpr>, Typ);

    fn make(egraph: &EGraph<StringExpr, Self>, enode: &StringExpr) -> Self::Data {
        let examples = egraph.analysis.examples;
        let outs = examples
            .iter()
            .map(|(inp, _)| {
                egg_lang::simplify(
                    enode
                        .build_recexpr(|id| egraph.id_to_expr(id).as_ref().last().unwrap().clone()),
                    inp,
                )
            })
            .collect::<Vec<_>>();

        use StringExpr::*;
        let typ = match outs[0] {
            Loc(_) | End | Add(_) | Find(_) => Typ::Loc,
            Lit(_) | Input | Concat(_) | Slice(_) => Typ::Str,
            Symbol(_) => Typ::Symbol,
        };

        (outs, typ)
    }

    fn merge(&mut self, a: &mut Self::Data, b: Self::Data) -> DidMerge {
        assert!(a.0.iter().zip_eq(b.0.iter()).all(|(a, b)| dbg!(a) == dbg!(b)));
        DidMerge(false, false)
    }
}

pub fn bottom_up_egg(
    examples: &[(String, String)],
) -> Option<(StringExpr, EGraph<StringExpr, ObservationalEquivalence>)> {
    use egg::Symbol;
    use StringExpr::{Symbol as Sym, *};

    let analysis = ObservationalEquivalence { examples };
    let mut bank = EGraph::new(analysis).with_explanations_enabled();
    bank.add(Sym(Symbol::new("Error")));
    bank.add(Loc(0));
    bank.add(Loc(1));
    bank.add(End);
    bank.add(Lit(" ".to_string()));
    bank.add(Input);
    bank.rebuild();

    for i in 0..10 {
        let mut adjs = {
            let strings = bank
                .classes()
                .filter(|class| {
                    let (_, typ) = class.data;
                    typ == Typ::Str
                })
                .collect::<Vec<_>>();

            let locs = bank
                .classes()
                .filter(|class| {
                    let (_, typ) = class.data;
                    typ == Typ::Loc
                })
                .collect::<Vec<_>>();

            dbg!(locs.iter().map(|c| &c.data.0).collect::<Vec<_>>());

            let loc_adds = locs
                .iter()
                .flat_map(|lhs| locs.iter().map(|rhs| StringExpr::Add([lhs.id, rhs.id])));

            let concats = strings.iter().flat_map(|lhs| {
                strings
                    .iter()
                    .map(|rhs| StringExpr::Concat([lhs.id, rhs.id]))
            });

            let finds = strings
                .iter()
                .flat_map(|lhs| strings.iter().map(|rhs| StringExpr::Find([lhs.id, rhs.id])));

            let slices = strings.iter().flat_map(|lhs| {
                locs.iter().flat_map(|start| {
                    locs.iter()
                        .map(|end| StringExpr::Slice([lhs.id, start.id, end.id]))
                })
            });

            slices
                .chain(concats)
                .chain(finds)
                .chain(loc_adds)
                .collect::<Vec<_>>()
        };
        //     .map(|prog| prog.simplify(&StringExpr::Input))
        //     .collect::<Vec<_>>();

        let old_len = if i == 0 { 0 } else { bank.number_of_classes() };
        for adj in adjs.drain(..) {
            // let exp = adj
            //             .build_recexpr(|id| bank.id_to_expr(id).as_ref().last().unwrap().clone());
            // let id = bank.add(adj.clone());
            let id = bank.add(adj);
            bank.rebuild();
            let (outs, _) = &bank[bank.find(id)].data;
            // dbg!(exp.to_string(), outs.iter().map(|x| x.to_string()).collect::<Vec<_>>());

            let equiv = bank.classes().find(|class| {
                let (class_outs, _) = &class.data;
                class_outs.iter().zip(outs.iter()).all(|(co, ao)| co == ao)
            });

            if let Some(class) = equiv {
                // dbg!((&class.data.0, outs));
                bank.union(bank.find(class.id), bank.find(id));
                bank.rebuild();
                // dbg!(bank.total_size(), bank.number_of_classes());
            }

            // let class = bank.lookup(adj);
            // dbg!(exp.to_string(), class.unwrap());
        }

        for new_id in old_len..bank.number_of_classes() {
            let (outs, _) = &bank[Id::from(new_id)].data;
            let correct = examples
                .iter()
                .zip(outs.iter())
                .all(|((_, expected_out), got_out)| {
                    expected_out == &got_out.to_string()
                });

            if correct {
                return Some((bank[Id::from(new_id)].nodes[0].clone(), bank));
            }
        }
    }

    None
}
