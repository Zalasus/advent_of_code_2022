
use std::str::FromStr;


type Priority = u32;


#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Item(char);

impl Item {
    fn priority(&self) -> Priority {
        match self.0 {
            'a'..='z' => (self.0 as u32) - ('a' as u32) + 1,
            'A'..='Z' => (self.0 as u32) - ('A' as u32) + 27,
            _ => panic!("Invalid item"),
        }
    }
}

impl TryFrom<char> for Item {
    type Error = char;
    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            'a'..='z' | 'A'..='Z' => Ok(Self(c)),
            _ => Err(c),
        }
    }
}


#[derive(Debug, PartialEq, Eq)]
struct Rucksack(Vec<Item>);

impl Rucksack {
    fn all(&self) -> &[Item] {
        &self.0
    }

    fn all_mut(&mut self) -> &mut [Item] {
        &mut self.0
    }

    fn compartments_mut(&mut self) -> (&mut [Item], &mut [Item]) {
        let mid = self.0.len() / 2;
        self.0.split_at_mut(mid)
    }

    fn find_common_item(&mut self) -> Item {
        let (left, right) = self.compartments_mut();
        right.sort_unstable();

        let mut common_item = None;
        for item in left {
            if right.binary_search(item).is_ok() {
                // item is present in both compartments
                if common_item.map(|i| i != *item).unwrap_or(false)  {
                    panic!("More than one common item type: {item:?}");
                }
                common_item = Some(*item);
            }
        }

        common_item.expect("No common item")
    }
}

#[derive(Debug, PartialEq, Eq)]
enum RucksackParseError {
    InvalidItemChar(char),
    OddItemCount,
}

impl FromStr for Rucksack {
    type Err = RucksackParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut items = Vec::new();
        for c in s.chars() {
            let item = Item::try_from(c).map_err(|e| RucksackParseError::InvalidItemChar(e))?;
            items.push(item);
        }

        if items.len() % 2 != 0 {
            Err(RucksackParseError::OddItemCount)
        } else {
            Ok(Self(items))
        }
    }
}


fn parse_input(input: &str) -> Vec<Rucksack> {
    input.split('\n')
        .filter(|line| !line.is_empty())
        .map(|line| Rucksack::from_str(line.trim()).expect("Malformed rucksack"))
        .collect()
}


fn solve_part1(input: &str) -> Priority {
    let rucksacks = parse_input(input);
    let mut total_prio = 0;
    for mut rucksack in rucksacks {
        total_prio += rucksack.find_common_item().priority();
    }
    total_prio
}

fn solve_part2(input: &str) -> Priority {
    let mut rucksacks = parse_input(input);
    let mut total_badge_prio = 0;
    for group in rucksacks.chunks_mut(3) {
        group[1].all_mut().sort_unstable();
        group[2].all_mut().sort_unstable();
        let mut badge = None;
        for item in group[0].all() {
            if group[1].all().binary_search(item).is_ok() {
                // candidate for badge
                if group[2].all().binary_search(item).is_ok() {
                    // found badge
                    if badge.map(|i| i != *item).unwrap_or(false)  {
                        panic!("More than one common item type: {item:?}");
                    }
                    badge = Some(*item);
                }
            }
        }
        total_badge_prio += badge.expect("No badge found").priority();
    }
    total_badge_prio
}


static INPUT: &str = include_str!("inputs/day3.txt");

pub fn run() {
    let part1 = solve_part1(INPUT);
    println!("Total priorities of common items in compartments: {part1}");

    let part2 = solve_part2(INPUT);
    println!("Total priorities of common items in groups of three rucksacks: {part2}");
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse() {
        assert_eq!(Item::try_from('a'), Ok(Item('a')));
        assert_eq!(Item::try_from('Z'), Ok(Item('Z')));
        assert_eq!(Item::try_from('#'), Err('#'));
        assert_eq!(Item::try_from('7'), Err('7'));

        let expected = &[ Item('a'), Item('b'), Item('c'), Item('d'), Item('e'), Item('f') ];
        assert_eq!(Rucksack::from_str("abcdef").unwrap().0, expected);
        assert_eq!(Rucksack::from_str("OwO what's this?"),
            Err(RucksackParseError::InvalidItemChar(' ')));
        assert_eq!(Rucksack::from_str("Ziebelzobel"), Err(RucksackParseError::OddItemCount));
    }

    fn check_common_item(rucksack_def: &str, expected_common_item: char) {
        let mut rucksack = Rucksack::from_str(rucksack_def).unwrap();
        assert_eq!(rucksack.find_common_item(), Item(expected_common_item));
    }

    #[test]
    fn common_item() {
        check_common_item("vJrwpWtwJgWrhcsFMMfFFhFp", 'p');
        check_common_item("jqHRNqRjqzjGDLGLrsFMfFZSrLrFZsSL", 'L');
        check_common_item("PmmdzqPrVvPwwTWBwg", 'P');
        check_common_item("wMqvLMZHhHMvwLHjbvcjnnSBnvTQFn", 'v');
        check_common_item("ttgJtRGJQctTZtZT", 't');
        check_common_item("CrZsJsPPZsGzwwsLwLmpwMDw", 's');
    }

    #[test]
    fn example_p1() {
        let input = "
            vJrwpWtwJgWrhcsFMMfFFhFp
            jqHRNqRjqzjGDLGLrsFMfFZSrLrFZsSL
            PmmdzqPrVvPwwTWBwg
            wMqvLMZHhHMvwLHjbvcjnnSBnvTQFn
            ttgJtRGJQctTZtZT
            CrZsJsPPZsGzwwsLwLmpwMDw";
        let part1 = solve_part1(input);
        assert_eq!(part1, 157);
    }

    #[test]
    fn example_p2() {
        let input = "
            vJrwpWtwJgWrhcsFMMfFFhFp
            jqHRNqRjqzjGDLGLrsFMfFZSrLrFZsSL
            PmmdzqPrVvPwwTWBwg
            wMqvLMZHhHMvwLHjbvcjnnSBnvTQFn
            ttgJtRGJQctTZtZT
            CrZsJsPPZsGzwwsLwLmpwMDw";
        let part2 = solve_part2(input);
        assert_eq!(part2, 70);
    }
}
