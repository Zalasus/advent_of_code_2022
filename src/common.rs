
use std::str::FromStr;


/// Simple tokenizer and parser for space-separated data.
///
/// My hate for regex burns with the passion of a thousand suns.
pub struct Words<'a>(&'a str);

impl<'a> Words<'a> {
    pub fn new(s: &'a str) -> Self {
        Self(s)
    }

    pub fn into_inner(self) -> &'a str {
        self.0
    }

    pub fn has_next(&self) -> bool {
        !self.0.is_empty()
    }

    pub fn next_word(&mut self) -> Result<&'a str, WordsError<'a>> {
        if self.0.is_empty() {
            Err(WordsError::Missing)
        } else if let Some((word, rest)) = self.0.split_once(' ') {
            self.0 = rest;
            Ok(word)
        } else {
            let word = self.0;
            self.0 = "";
            Ok(word)
        }
    }

    pub fn expect_word(&mut self, expected: &str) -> Result<(), WordsError<'a>> {
        let word = self.next_word()?;
        if word == expected {
            Ok(())
        } else {
            Err(WordsError::Unexpected(word))
        }
    }

    pub fn next_parsed<T: FromStr>(&mut self) -> Result<T, WordsError<'a>> {
        let word = self.next_word()?;
        word.parse().map_err(|_| WordsError::Unparsable(word))
    }

    pub fn expect_sentence(&mut self, words: &[&str]) -> Result<(), WordsError<'a>> {
        for word in words {
            self.expect_word(word)?;
        }
        Ok(())
    }
}

pub enum WordsError<'a> {
    Missing,
    Unexpected(&'a str),
    Unparsable(&'a str),
}


pub fn parse_separated_list<T: FromStr>(input: &str, separator: char) -> Result<Vec<T>, T::Err> {
    let items_estimate = input.chars().filter(|c| *c == separator).count() + 1;
    let mut list = Vec::with_capacity(items_estimate);
    for item_str in input.split(separator) {
        list.push(item_str.trim().parse()?);
    }
    Ok(list)
}


/// A blanket trait extending slices with the ability to acquire multiple mutable references to
/// distinct indices within.
pub trait GetMuts<'a> {
    type Item;

    /// Returns multiple mutable references to the N distinct indices.
    ///
    /// This will panic if indices are not unique. Indices are also bounds checked. If an index is
    /// out of bounds, this will panic.
    fn get_muts<const N: usize>(self, indices: [usize; N])
        -> [&'a mut Self::Item; N];
}

impl<'a, T> GetMuts<'a> for &'a mut [T] {
    type Item = T;

    fn get_muts<const N: usize>(self, indices: [usize; N]) -> [&'a mut T; N] {
        let len = self.len();
        let idx_range = 0..len;
        for idx in indices {
            if !idx_range.contains(&idx) {
                panic!("Index {idx} out of bounds (len = {len})");
            }
        }

        let mut indices_unique_check = indices;
        indices_unique_check.sort_unstable();
        if indices_unique_check.windows(2).any(|w| w[0] == w[1]) {
            panic!("Non-unique indices");
        }

        let ptr = self.as_mut_ptr();
        std::array::from_fn(|idx| {
            let src_idx = indices[idx];
            let element = ptr.wrapping_offset(src_idx.try_into().unwrap());

            // SAFETY: we checked that src_idx is in range, and that all indices in the array
            // are distinct. we ascribe to the reference the lifetime of the slice ptr points
            // into, so the resulting borrow can't outlive the slice it references.
            unsafe {
                &mut *element
            }
        })
    }
}
