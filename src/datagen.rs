use rand::Rng;
use core::marker::ConstParamTy;

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

pub fn mk_programs(limit: usize) {
    // ideally lazy iterator of infinite programs by size
    // https://doc.rust-lang.org/nightly/std/ops/trait.Coroutine.html -- maybe too unstable
    // maybe use from_fn
    // maybe a struct impl iterator
    todo!()
}
