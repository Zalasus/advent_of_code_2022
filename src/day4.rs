
use std::array;
use std::str::FromStr;


/// A range of campground IDs.
#[derive(Debug, PartialEq, Eq)]
struct IdRange {
    start: usize,
    end: usize,
}

impl IdRange {
    /// `start` and `end` are inclusive.
    fn new(start: usize, end_inclusive: usize) -> Self {
        Self {
            start,
            end: end_inclusive + 1,
        }
    }

    fn contains(&self, point: usize) -> bool {
        point >= self.start && point < self.end
    }

    fn contains_range(&self, other: &Self) -> bool {
        self.start <= other.start && self.end >= other.end
    }

    fn overlaps_range(&self, other: &Self) -> bool {
        other.contains(self.start) || other.contains(self.end - 1)
    }
}

#[derive(Debug)]
enum IdRangeParseError {
    Separator,
    ParseInt,
}

impl FromStr for IdRange {
    type Err = IdRangeParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (start_str, end_str) = s.split_once('-').ok_or(IdRangeParseError::Separator)?;
        let parse_error = |_| IdRangeParseError::ParseInt;
        let start = start_str.parse().map_err(parse_error)?;
        let end = end_str.parse().map_err(parse_error)?;
        Ok(Self::new(start, end))
    }
}

fn parse_input(input: &str) -> Vec<[IdRange; 2]> {
    input.split('\n')
        .filter(|l| !l.is_empty())
        .map(|l| {
            let mut parts = l.trim().split(',');
            array::from_fn(|_| IdRange::from_str(parts.next().unwrap()).unwrap())
        })
        .collect()
}

fn count_ranges<F>(input: &str, mut f: F) -> usize
where
    F: FnMut(&IdRange, &IdRange) -> bool
{
    let parsed = parse_input(input);
    parsed.iter()
        .filter(|pair| f(&pair[0], &pair[1]) || f(&pair[1], &pair[0]))
        .count()
}


static INPUT: &str = include_str!("inputs/day4.txt");

pub fn run() {
    let enclosed = count_ranges(INPUT, IdRange::contains_range);
    println!("Completely enclosed ranges: {enclosed}");

    let overlapping = count_ranges(INPUT, IdRange::overlaps_range);
    println!("Overlapping ranges: {overlapping}");
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn example() {
        let input = "
            2-4,6-8
            2-3,4-5
            5-7,7-9
            2-8,3-7
            6-6,4-6
            2-6,4-8";
        let parsed = parse_input(input);
        assert_eq!(parsed[0][0], IdRange::new(2, 4));
        assert_eq!(parsed[0][1], IdRange::new(6, 8));
        assert_eq!(parsed[3][1], IdRange::new(3, 7));
        let enclosed = count_ranges(input, IdRange::contains_range);
        assert_eq!(enclosed, 2);

        let overlapping = count_ranges(input, IdRange::overlaps_range);
        assert_eq!(overlapping, 4);
    }
}
