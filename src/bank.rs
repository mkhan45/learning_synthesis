use std::mem::MaybeUninit;

pub struct Bank<T> {
    pub entries: Vec<Vec<T>>,
}

impl<T> Bank<T> {
    pub fn new() -> Self {
        Bank { entries: Vec::new() }
    }

    pub fn grow_to(&mut self, size: usize) {
        if self.entries.len() < size {
            self.entries.extend((self.entries.len()..=size).map(|_| Vec::new()))
        }
    }

    pub fn size(&self, size: usize) -> &Vec<T> {
        self.grow_to(size);
        &self.entries[size - 1]
    }

    pub fn size_mut(&mut self, size: usize) -> &mut Vec<T> {
        self.grow_to(size);
        &mut self.entries[size - 1]
    }
}

pub struct BankIterator<T> {
    pub size: usize,
    pub index: usize,
    pub bank: Bank<MaybeUninit<T>>,
}

impl<T> Iterator for BankIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.size > self.bank.entries.len() {
            None
        } else if self.index >= self.bank.entries[self.size].len() {
            self.size += 1;
            self.index = 0;
            self.next()
        } else {
            let res = std::mem::replace(
                &mut self.bank.entries[self.size][self.index],
                MaybeUninit::uninit(),
            );
            self.index += 1;
            unsafe { Some(res.assume_init()) }
        }
    }
}

impl<T> IntoIterator for Bank<T> {
    type Item = T;
    type IntoIter = BankIterator<T>;

    fn into_iter(self) -> Self::IntoIter {
        BankIterator {
            size: 0,
            index: 0,
            bank: Bank {
                entries: self
                    .entries
                    .into_iter()
                    .map(|v| v.into_iter().map(|x| MaybeUninit::new(x)).collect())
                    .collect(),
            },
        }
    }
}
