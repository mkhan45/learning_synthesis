use rand::Rng;
use core::marker::ConstParamTy;

use std::fs::File;
use std::path::Path;
use std::io::{BufWriter, Write};

use itertools::Itertools;

use crate::vsa::{Lit, Fun, AST};

type Program = AST<Lit, Fun>;

use im::Vector as iVec;

// could be a type but then I need phantom data
#[derive(ConstParamTy, PartialEq, Eq)]
pub enum StringRNGToken {
    Test,
}

pub struct StringRNG<const ID: StringRNGToken> {
    pub dict: Vec<String>,
    pub punct: Vec<String>,
    pub rng: rand::rngs::ThreadRng,
}

impl<const ID: StringRNGToken> StringRNG<ID> {
    pub fn new(dict: Vec<String>, punct: Vec<String>) -> Self {
        Self {
            dict,
            punct,
            rng: rand::thread_rng(),
        }
    }

    pub fn stringify_str(&self, str: &TokString<ID>) -> String {
        let toks = str.0.iter();
        let followed_by_punct = toks.clone().skip(1).map(|tok| match tok {
            Token::Punct(_) => true,
            _ => false,
        }).chain(std::iter::once(true));

        let mut builder = String::new();
        for (tok, followed_by_punct) in toks.zip(followed_by_punct) {
            match tok {
                Token::Word { idx, is_cap } => {
                    let s = &self.dict[*idx];
                    if *is_cap {
                        builder.push_str(&s[..1].to_uppercase());
                        builder.push_str(&s[1..]);
                    } else {
                        builder.push_str(&s);
                    }
                }
                Token::Punct(idx) => builder.push_str(&self.punct[*idx]),
            }

            if !followed_by_punct {
                builder.push(' ');
            }
        }

        builder
    }
}

#[derive(Clone, Copy)]
pub enum Token {
    Word { idx: usize, is_cap: bool },
    Punct(usize),
}

// in the future, might want a tree structure
// ideally, LLM generate without accidentally stealing from the test set
pub struct TokString<const ID: StringRNGToken>(Vec<Token>);
impl<const ID: StringRNGToken> TokString<ID> {
    pub fn to_string(&self, bank: &StringRNG<ID>) -> String {
        bank.stringify_str(self)
    }
}

impl<const ID: StringRNGToken> StringRNG<ID> {
    fn new_word(&mut self) -> Token {
        Token::Word {
            idx: self.rng.gen_range(0..self.dict.len()),
            is_cap: self.rng.gen_bool(0.2),
        }
    }

    fn new_punct(&mut self) -> Token {
        Token::Punct(self.rng.gen_range(0..self.punct.len()))
    }

    pub fn gen_string(&mut self, size: usize) -> TokString<ID> {
        let mut toks = Vec::new();
        toks.push(self.new_word());
        let mut last_n_words = 1;
        for _ in 1..size {
            let is_punct = self.rng.gen_bool((0.3 * last_n_words as f64).min(1.0));
            if is_punct {
                toks.push(self.new_punct());
                last_n_words = 0;
            } else {
                toks.push(self.new_word());
                last_n_words += 1;
            }
        }
        TokString(toks)
    }

    pub fn gen_string_fr(&mut self, size: usize) -> String {
        self.gen_string(size).to_string(self)
    }
}

pub struct ProgramGen {
    pub bank: Vec<Vec<Program>>,
    pub current_arity: usize,
    pub current_size: usize,
    pub ops: Vec<Vec<Fun>>,

    size_arity_iter: Box<dyn Iterator<Item = (usize, Vec<usize>)>>,
}

// should be iterator
pub fn sum_permutations(n: usize, target: usize) -> Vec<Vec<usize>> {
    let mut res = Vec::new();
    let mut current = vec![0; n];
    let mut i = 0;
    while i < n {
        if current[i] < target {
            current[i] += 1;
            if current.iter().sum::<usize>() == target {
                res.push(current.clone());
            } else {
                i = 0;
            }
        } else {
            current[i] = 0;
            i += 1;
        }
    }
    res
}

#[test]
fn test_sum_permutations() {
    let res = sum_permutations(3, 9);
    assert!(res.iter().all(|v| v.iter().sum::<usize>() == 9));
    assert!(res.iter().all(|v| v.iter().count() == 3));
    assert!(res.iter().any(|v| v == &[3, 3, 3]));
    assert!(res.iter().any(|v| v == &[1, 5, 3]));
    assert!(res.iter().any(|v| v == &[3, 5, 1]));
}


// doesnt need a macro but i want to leak!
macro_rules! leak {
    ($e:expr) => { Box::leak(Box::new($e)) };
}

impl ProgramGen {
    pub fn size_arity_iter<'a>(
        bank_sizes: &'a [usize], arity_lens: &'a [usize], arity: usize, size: usize
    ) -> Box<dyn Iterator<Item = (usize, Vec<usize>)> + 'a> {
        let child_arg_sizes = sum_permutations(arity, size-1);
        let all_children = child_arg_sizes.into_iter().map(|v| {
            // v is the sizes needed for each child branch
            // we get a [[ChildIdx; Arity]; NumChildren]
            v.into_iter().flat_map(|csize| (0..bank_sizes[csize-1]))
        });

        // for each op of the right arity, iter through every child size combo
        let app_op_is = 0..arity_lens[arity];
        Box::new(app_op_is.flat_map(move |op_i| {
            all_children.clone().map(move |children_is| (op_i, children_is.collect()))
        }))
    }

    pub fn new(start_bank: Vec<Program>, ops: Vec<Vec<Fun>>) -> Self {
        let bank = vec![start_bank, vec![]];
        let current_arity = 1;
        let current_size = 2;
        let bank_sizes = leak!([bank[0].len()]);
        let arity_lens = leak!(ops.iter().map(|v| v.len()).collect::<Vec<_>>());
        let size_arity_iter = Self::size_arity_iter(bank_sizes, arity_lens, current_arity, current_size);

        Self {
            bank,
            current_arity,
            current_size,
            ops,
            size_arity_iter,
        }
    }
}

impl Iterator for ProgramGen {
    type Item = Program;

    fn next(&mut self) -> Option<Self::Item> {
        match self.size_arity_iter.next() {
            Some((op_i, children_is)) => {
                let op = self.ops[self.current_arity][op_i];
                // should ideally use a hashcons'd AST
                let children = children_is.into_iter().map(|child_i| {
                    let size_bank = &self.bank[self.current_size - 2];
                    size_bank[child_i].clone()
                }).collect();
                Some( AST::App { fun: op, args: children } )
            }
            None => {
                if self.current_arity == self.ops.len() {
                    self.current_arity = 1;
                    self.current_size += 1;
                    self.bank.push(Vec::new());
                    if self.current_size > self.bank.len() {
                        return None;
                    }
                }
                self.size_arity_iter = Self::size_arity_iter(
                    leak!(self.bank.iter().map(|v| v.len()).collect::<Vec<_>>()),
                    leak!(self.ops.iter().map(|v| v.len()).collect::<Vec<_>>()),
                    self.current_arity,
                    self.current_size
                    );
                self.next()
            }
        }
    }
}

pub struct Examples<'a, const T: StringRNGToken> {
    pub prog: &'a Program,
    pub inps: &'a [String],
}

// pub struct TraceApp {
//     fun: Fun,
//     args: Vec<Lit>,
// }

// pub struct Trace {
//     pub out: Lit,
//     pub from: Option<TraceApp>,
// }

// just a cached AST but feels better bc it's upside down
// and the funs are conceptually inverse
// lowkey a singleton vsa
pub enum Trace {
    Final(Lit),
    App { out: Lit, fun: Fun, args: Vec<Trace> }
}

impl Trace {
    pub fn value<'a>(&'a self) -> &'a Lit {
        match self {
            Trace::Final(l) => l,
            Trace::App { out, .. } => out,
        }
    }

    pub fn value_clone(&self) -> Lit {
        self.value().clone()
    }
}

impl<'a, const T: StringRNGToken> Examples<'a, T> {
    pub fn new(prog: &'a Program, inps: &'a [String]) -> Self {
        Examples { prog, inps }
    }

    // might be good to hashcons
    // i feel like this is stupid and I should just
    // do it in one pass
    // pub fn traces(&self) -> Vec<Trace> {
    //     todo!()
    // }
    pub fn write_traces<P: AsRef<Path>>(&mut self, path: P) -> std::io::Result<()> {
        let f = File::create(path)?;
        let mut writer = BufWriter::new(f);
        
        for inp in self.inps {
            let inp_lit = Lit::StringConst(inp.clone());
            let out = self.prog.eval(&inp_lit);
            let first_trace = im::vector![format!("{} → {};", inp_lit, out)];
            Self::trace(first_trace, self.prog, &inp_lit, &mut writer)?;
        }

        writer.flush()?;
        Ok(())
    }

    fn trace(parent_trace: iVec<String>, prog: &'a Program, inp: &Lit, writer: &mut BufWriter<File>) -> std::io::Result<()> {
        // 1. dfs program trace, add stuff to imlist
        // 2. at the end of each path, write stuff from imlist, it was reversed

        let out = prog.eval(&inp);
        match prog {
            AST::Lit(l) => {
                // weird lib
                let mut final_trace = parent_trace.clone();
                final_trace.push_back(format!("{} ← Lit({})", out, l));

                for trace_line in final_trace.iter() {
                    write!(writer, "{}\n", trace_line)?;
                }
                write!(writer, "\n")?;
            }
            AST::App { fun, args } => {
                let mut arg_vals = args.iter().map(|c| c.eval(inp));
                let arg_str = arg_vals.join(", ");
                let mut next_trace = parent_trace.clone();
                next_trace.push_back(format!("{} ← {:?}({})", out, fun, arg_str));
                for arg in args {
                    Self::trace(next_trace.clone(), arg, inp, writer)?;
                }
            }
        }

        Ok(())
    }
}

impl std::fmt::Display for Lit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Lit::StringConst(s) => {
                f.write_str("\"")?; 
                f.write_str(s)?;
                f.write_str("\"")?; 
                Ok(())
            }
            Lit::LocConst(l) => f.write_str(&l.to_string()),
            Lit::BoolConst(b) => f.write_str(&b.to_string()),
            Lit::LocEnd => f.write_str("$"),
            Lit::Input => f.write_str("X"),
        }
    }
}
