
//! This one's probably a bit overdone, but the most correct solution I could come up with.

use crate::common::{Words, WordsError, GetMuts};

use std::str::FromStr;


#[derive(Debug, PartialEq, Eq)]
struct Instruction {
    count: usize,
    from: usize,
    to: usize,
}

#[derive(Debug, PartialEq, Eq)]
enum InstructionParseError {
    MissingPart,
    BadKeyword,
    BadNumber,
    TrailingWords,
}

impl From<WordsError<'_>> for InstructionParseError {
    fn from(e: WordsError<'_>) -> Self {
        match e {
            WordsError::Missing => Self::MissingPart,
            WordsError::Unexpected(_) => Self::BadKeyword,
            WordsError::Unparsable(_) => Self::BadNumber,
        }
    }
}

impl FromStr for Instruction {
    type Err = InstructionParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut words = Words::new(s);
        words.expect_word("move")?;
        let count = words.next_parsed()?;
        words.expect_word("from")?;
        let from = words.next_parsed()?;
        words.expect_word("to")?;
        let to = words.next_parsed()?;

        if words.has_next() {
            return Err(InstructionParseError::TrailingWords);
        }

        Ok(Self {
            count,
            from,
            to,
        })
    }
}


struct CrateRowIterator<'a>(std::str::Chars<'a>);

impl<'a> CrateRowIterator<'a> {
    fn new(s: &'a str) -> Self {
        Self(s.chars())
    }
}

impl Iterator for CrateRowIterator<'_> {
    type Item = Crate;

    fn next(&mut self) -> Option<Self::Item> {
        // first character may be None. in that case, iterator is at end.
        let first = self.0.next()?;

        let mut crate_spec = [first, '\0', '\0'];
        for c in &mut crate_spec[1..] {
            *c = if let Some(c) = self.0.next() {
                c
            } else {
                return Some(Crate::Error(CrateError::MissingCharacter));
            };
        }

        // check and consume crate separator
        match self.0.next() {
            Some(' ') | None => (),
            Some(_) => return Some(Crate::Error(CrateError::BadTrailingCharacter)),
        }

        match crate_spec {
            [' ', ' ', ' '] => Some(Crate::Missing),
            ['[', sym, ']'] => Some(Crate::Labeled(sym)),
            _ => Some(Crate::Error(CrateError::BadCrateSpec)),
        }
    }
}


#[derive(Debug, PartialEq, Eq)]
enum CrateError {
    MissingCharacter,
    BadCrateSpec,
    BadTrailingCharacter,
}

#[derive(Debug, PartialEq, Eq)]
enum Crate {
    Missing,
    Labeled(char),
    Error(CrateError),
}


fn parse_input(input: &str) -> (Vec<Vec<char>>, Vec<Instruction>) {
    let (stacks_str, instructions_str) = input.split_once("\n\n").unwrap();

    // parse stacks, starting from the bottom
    let mut stack_lines = stacks_str.rsplit('\n');
    let number_row = stack_lines.next().unwrap();
    let columns = number_row.trim().split("   ").count();
    let mut stacks = vec![Vec::new(); columns];
    for line in stack_lines {
        for (crate_column, crate_label) in CrateRowIterator::new(line).enumerate() {
            match crate_label {
                Crate::Missing => (),
                Crate::Labeled(label) => stacks[crate_column].push(label),
                Crate::Error(e) => panic!("Crate parse error {e:?}"),
            }
        }
    }

    let instructions = instructions_str.lines()
        .map(|line| Instruction::from_str(line).unwrap())
        .collect();

    (stacks, instructions)
}


enum CraneModel {
    CrateMover9000,
    CrateMover9001,
}

fn run_freightyard(input: &str, crane: CraneModel) -> String {
    let (mut stacks, instructions) = parse_input(input);

    for instruction in instructions {
        let count = instruction.count;

        let [from, to] = stacks.get_muts([instruction.from - 1, instruction.to - 1]);

        let moved_stack = from.drain((from.len()-count)..);

        match crane {
            CraneModel::CrateMover9000 => to.extend(moved_stack.rev()),
            CraneModel::CrateMover9001 => to.extend(moved_stack),
        }
    }

    stacks.iter().filter_map(|stack| stack.last()).cloned().collect()
}


static INPUT: &str = include_str!("inputs/day5.txt");

pub fn run() {
    let part1 = run_freightyard(INPUT, CraneModel::CrateMover9000);
    println!("Top crates using CrateMover 9000: {}", part1);

    let part2 = run_freightyard(INPUT, CraneModel::CrateMover9001);
    println!("Top crates using CrateMover 9001: {}", part2);
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn stack_parsing() {
        let input = "[A]     [Ü] [漢]";
        let mut iter = CrateRowIterator::new(input);
        assert_eq!(iter.next(), Some(Crate::Labeled('A')));
        assert_eq!(iter.next(), Some(Crate::Missing));
        assert_eq!(iter.next(), Some(Crate::Labeled('Ü')));
        assert_eq!(iter.next(), Some(Crate::Labeled('漢')));
        assert_eq!(iter.next(), None);

        assert_eq!(CrateRowIterator::new("[F").next(),
            Some(Crate::Error(CrateError::MissingCharacter)));

        assert_eq!(CrateRowIterator::new("---").next(),
            Some(Crate::Error(CrateError::BadCrateSpec)));

        assert_eq!(CrateRowIterator::new("[O]+").next(),
            Some(Crate::Error(CrateError::BadTrailingCharacter)));
    }

    #[test]
    fn instruction_parsing() {
        let input = "move 10 from 5 to 0";
        let expected = Instruction {
            count: 10,
            from: 5,
            to: 0,
        };
        assert_eq!(Instruction::from_str(input), Ok(expected));

        assert_eq!(Instruction::from_str("foo"),
            Err(InstructionParseError::BadKeyword));

        assert_eq!(Instruction::from_str("move zero"),
            Err(InstructionParseError::BadNumber));

        assert_eq!(Instruction::from_str("move 0"),
            Err(InstructionParseError::MissingPart));

        assert_eq!(Instruction::from_str("move 0 from 1 to 2 please"),
            Err(InstructionParseError::TrailingWords));
    }

    #[test]
    fn example() {
        let input = concat!(
            "    [D]    \n",
            "[N] [C]    \n",
            "[Z] [M] [P]\n",
            " 1   2   3 \n",
            "\n",
            "move 1 from 2 to 1\n",
            "move 3 from 1 to 3\n",
            "move 2 from 2 to 1\n",
            "move 1 from 1 to 2\n",
        );

        let (stacks, instructions) = parse_input(input);

        assert_eq!(stacks[0], &['Z', 'N']);
        assert_eq!(stacks[1], &['M', 'C', 'D']);
        assert_eq!(stacks[2], &['P']);

        assert_eq!(instructions[0].count, 1);
        assert_eq!(instructions[0].from, 2);
        assert_eq!(instructions[0].to, 1);
        assert_eq!(instructions[3].count, 1);
        assert_eq!(instructions[3].from, 1);
        assert_eq!(instructions[3].to, 2);

        let part1 = run_freightyard(input, CraneModel::CrateMover9000);
        assert_eq!(part1, "CMZ");

        let part2 = run_freightyard(input, CraneModel::CrateMover9001);
        assert_eq!(part2, "MCD");
    }
}
