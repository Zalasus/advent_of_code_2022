
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


/// Gets mutable references to two positions in a slice as long as they are distinct.
///
/// Not sure why this is not in the core library.
pub fn get_two_mut<T>(slice: &mut [T], a: usize, b: usize) -> (&mut T, &mut T) {
    use std::cmp::Ordering;
    match a.cmp(&b) {
        Ordering::Less => {
            let (left, right) = slice.split_at_mut(b);
            (&mut left[a], &mut right[0])
        },
        Ordering::Greater => {
            let (left, right) = slice.split_at_mut(a);
            (&mut right[0], &mut left[b])
        },
        Ordering::Equal => panic!("Non-unique indices"),
    }
}


pub fn parse_separated_list<T: FromStr>(input: &str, separator: char) -> Result<Vec<T>, T::Err> {
    let items_estimate = input.chars().filter(|c| *c == separator).count() + 1;
    let mut list = Vec::with_capacity(items_estimate);
    for item_str in input.split(separator) {
        list.push(item_str.parse()?);
    }
    Ok(list)
}
