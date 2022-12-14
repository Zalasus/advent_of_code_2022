
use crate::common::{parse_separated_list, GetMuts};

use std::str::FromStr;


type WorryLevel = u64;


#[derive(Debug)]
enum MonkeyParseError {
    NumberParse,
    UnrecognizedLine,
    MissingPart,
    UnrecognizedOperator,
}


#[derive(Debug, PartialEq, Eq)]
enum Operand {
    Constant(WorryLevel),
    Old,
}

impl Operand {
    fn evaluate(&self, old: WorryLevel) -> WorryLevel {
        match self {
            Self::Constant(c) => *c,
            Self::Old => old,
        }
    }
}

impl FromStr for Operand {
    type Err = MonkeyParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "old" {
            Ok(Self::Old)
        } else {
            Ok(Self::Constant(s.parse().map_err(|_| MonkeyParseError::NumberParse)?))
        }
    }
}


#[derive(Debug, PartialEq, Eq)]
enum OperationKind {
    Add,
    Multiply,
}

impl OperationKind {
    fn evaluate(&self, lhs: WorryLevel, rhs: WorryLevel) -> WorryLevel {
        match self {
            Self::Add => lhs + rhs,
            Self::Multiply => lhs * rhs,
        }
    }
}

impl FromStr for OperationKind {
    type Err = MonkeyParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "+" => Ok(Self::Add),
            "*" => Ok(Self::Multiply),
            _ => Err(MonkeyParseError::UnrecognizedOperator),
        }
    }
}



#[derive(Debug, PartialEq, Eq)]
struct Operation {
    lhs: Operand,
    kind: OperationKind,
    rhs: Operand,
}

impl Operation {
    fn evaluate(&self, old: WorryLevel) -> WorryLevel {
        let lhs_value = self.lhs.evaluate(old);
        let rhs_value = self.rhs.evaluate(old);
        self.kind.evaluate(lhs_value, rhs_value)
    }
}

impl FromStr for Operation {
    type Err = MonkeyParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let expr = s.trim().strip_prefix("new = ").ok_or(MonkeyParseError::MissingPart)?;
        let mut words = expr.split(' ');
        let lhs = words.next()
            .ok_or(MonkeyParseError::MissingPart)?
            .parse()?;
        let kind = words.next()
            .ok_or(MonkeyParseError::MissingPart)?
            .parse()?;
        let rhs = words.next()
            .ok_or(MonkeyParseError::MissingPart)?
            .parse()?;
        Ok(Self {
            lhs,
            kind,
            rhs,
        })
    }
}


#[derive(Debug)]
struct MonkeyDef {
    id: usize,
    starting_items: Vec<WorryLevel>,
    operation: Operation,
    divisible_test: WorryLevel,
    true_monkey: usize,
    false_monkey: usize,
}

impl FromStr for MonkeyDef {
    type Err = MonkeyParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut id = None;
        let mut items = None;
        let mut operation = None;
        let mut test = None;
        let mut true_monkey = None;
        let mut false_monkey = None;
        for line in s.lines().map(str::trim) {
            if let Some(id_str) = line.strip_prefix("Monkey ") {
                id = Some(id_str.strip_suffix(':')
                        .ok_or(MonkeyParseError::MissingPart)?
                        .parse()
                        .map_err(|_| MonkeyParseError::NumberParse)?);
            } else if let Some(items_str) = line.strip_prefix("Starting items:") {
                items = Some(parse_separated_list(items_str, ',')
                        .map_err(|_| MonkeyParseError::NumberParse)?);
            } else if let Some(operation_str) = line.strip_prefix("Operation:") {
                operation = Some(Operation::from_str(operation_str)?);
            } else if let Some(test_str) = line.strip_prefix("Test: divisible by ") {
                test = Some(test_str.parse().map_err(|_| MonkeyParseError::NumberParse)?);
            } else if let Some(true_str) = line.strip_prefix("If true: throw to monkey ") {
                true_monkey = Some(true_str.parse().map_err(|_| MonkeyParseError::NumberParse)?);
            } else if let Some(false_str) = line.strip_prefix("If false: throw to monkey ") {
                false_monkey = Some(false_str.parse().map_err(|_| MonkeyParseError::NumberParse)?);
            } else {
                return Err(MonkeyParseError::UnrecognizedLine);
            }
        }

        Ok(Self {
            id: id.ok_or(MonkeyParseError::MissingPart)?,
            starting_items: items.ok_or(MonkeyParseError::MissingPart)?,
            operation: operation.ok_or(MonkeyParseError::MissingPart)?,
            divisible_test: test.ok_or(MonkeyParseError::MissingPart)?,
            true_monkey: true_monkey.ok_or(MonkeyParseError::MissingPart)?,
            false_monkey: false_monkey.ok_or(MonkeyParseError::MissingPart)?,
        })
    }
}


fn parse_input(input: &str) -> Vec<MonkeyDef> {
    let mut monkeys = input.split("\n\n")
        .map(|s| MonkeyDef::from_str(s).unwrap())
        .collect::<Vec<_>>();
    monkeys.sort_unstable_by_key(|m| m.id);
    monkeys
}


#[derive(Debug, Clone)]
struct Monkey<'a> {
    def: &'a MonkeyDef,
    items: Vec<WorryLevel>,
    inspected_item_count: usize,
}

impl<'a> Monkey<'a> {
    fn new(def: &'a MonkeyDef) -> Self {
        Self {
            def,
            items: def.starting_items.clone(),
            inspected_item_count: 0,
        }
    }
}


/// Simulates a single round of monkey shenanigans.
fn step_monkeys<F>(monkeys: &mut [Monkey<'_>], relief_function: &mut F)
where
    F: FnMut(WorryLevel) -> WorryLevel,
{
    for current_idx in 0..monkeys.len() {
        // borrow all thre monkeys involved
        let true_idx = monkeys[current_idx].def.true_monkey;
        let false_idx = monkeys[current_idx].def.false_monkey;
        let [current_monkey, true_monkey, false_monkey] = monkeys
            .get_muts([current_idx, true_idx, false_idx]);

        // a monkey always inspects all it's items
        current_monkey.inspected_item_count += current_monkey.items.len();

        for item in current_monkey.items.drain(..) {
            let inspected_item = current_monkey.def.operation.evaluate(item);
            let tested_item = relief_function(inspected_item);

            if tested_item % current_monkey.def.divisible_test == 0 {
                true_monkey.items.push(tested_item);
            } else {
                false_monkey.items.push(tested_item);
            }
        }
    }
}


fn top_most_active_monkeys<F>(input: &[MonkeyDef], rounds: usize, mut relief_function: F) -> usize
where
    F: FnMut(WorryLevel) -> WorryLevel,
{
    let mut monkeys = input.iter().map(Monkey::new).collect::<Vec<_>>();

    for _ in 0..rounds {
        step_monkeys(&mut monkeys, &mut relief_function);
    }

    monkeys.sort_unstable_by_key(|m| m.inspected_item_count);
    monkeys.iter().rev().take(2).map(|m| m.inspected_item_count).product()
}


fn calc_part_one(input: &[MonkeyDef]) -> usize {
    top_most_active_monkeys(input, 20, |worry| worry / 3)
}

fn calc_part_two(input: &[MonkeyDef]) -> usize {
    // without dividing by three, worry levels exceed managable levels. however, for 0 ??? i ??? n:
    //  a ??? 0 mod m_i ??? a ??? 0 mod lcm(m_0, .. m_n)
    // (according to wikipedia) a set of divisibility rules still holds true if one replaces the
    // moduli with their least common multiple. that is probably a wobbly definition and less than
    // half of the proof needed here, but i've been stuck to long with this and i wanna continue
    // even though i don't fully understand what's going on (yet).
    let div_test_lcm = input.iter()
        .map(|monkey| monkey.divisible_test)
        .reduce(num::integer::lcm)
        .unwrap();
    top_most_active_monkeys(input, 10000, |worry| worry % div_test_lcm)
}


static INPUT: &str = include_str!("inputs/day11.txt");

pub fn run() {
    let monkey_defs = parse_input(INPUT);
    let part1 = calc_part_one(&monkey_defs);
    println!("Items handled by top two active monkeys, multiplied together: {part1}");

    let part2 = calc_part_two(&monkey_defs);
    println!("Same, but without relief: {part2}");
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn monkey_parse() {
        let input = "Monkey 0:
                       Starting items: 79, 98
                       Operation: new = old * 19
                       Test: divisible by 23
                         If true: throw to monkey 2
                         If false: throw to monkey 3";
        let monkey: MonkeyDef = input.parse().unwrap();
        assert_eq!(monkey.id, 0);
        assert_eq!(monkey.starting_items, &[79, 98]);
        let expected_op = Operation {
            lhs: Operand::Old,
            kind: OperationKind::Multiply,
            rhs: Operand::Constant(19),
        };
        assert_eq!(monkey.operation, expected_op);
        assert_eq!(monkey.divisible_test, 23);
        assert_eq!(monkey.true_monkey, 2);
        assert_eq!(monkey.false_monkey, 3);
    }

    #[test]
    fn example() {
        let input = "Monkey 0:
                       Starting items: 79, 98
                       Operation: new = old * 19
                       Test: divisible by 23
                         If true: throw to monkey 2
                         If false: throw to monkey 3

                     Monkey 1:
                       Starting items: 54, 65, 75, 74
                       Operation: new = old + 6
                       Test: divisible by 19
                         If true: throw to monkey 2
                         If false: throw to monkey 0

                     Monkey 2:
                       Starting items: 79, 60, 97
                       Operation: new = old * old
                       Test: divisible by 13
                         If true: throw to monkey 1
                         If false: throw to monkey 3

                     Monkey 3:
                       Starting items: 74
                       Operation: new = old + 3
                       Test: divisible by 17
                         If true: throw to monkey 0
                         If false: throw to monkey 1";
        let parsed = parse_input(input);
        let part1 = calc_part_one(&parsed);
        assert_eq!(part1, 10605);

        let part2 = calc_part_two(&parsed);
        assert_eq!(part2, 2713310158);
    }


    /// Checks that two relief functions, f1 and f2, lead to the same monkey business within the
    /// given number of rounds.
    fn check_relief_equivalence<F1, F2>(input: &[MonkeyDef], max_rounds: usize, mut f1: F1,
        mut f2: F2) -> bool
    where
        F1: FnMut(WorryLevel) -> WorryLevel,
        F2: FnMut(WorryLevel) -> WorryLevel,
    {
        let mut monkeys_1 = input.iter().map(Monkey::new).collect::<Vec<_>>();
        let mut monkeys_2 = monkeys_1.clone();

        for _ in 0..max_rounds {
            step_monkeys(&mut monkeys_1, &mut f1);
            step_monkeys(&mut monkeys_2, &mut f2);

            let equivalent = monkeys_1.iter()
                .zip(monkeys_2.iter())
                .all(|(m1, m2)| m1.inspected_item_count == m2.inspected_item_count);
            if !equivalent {
                return false;
            }
        }

        true
    }

    #[test]
    fn relief_equivalence() {
        let input = "Monkey 0:
                       Starting items: 79, 98
                       Operation: new = old * 19
                       Test: divisible by 24
                         If true: throw to monkey 2
                         If false: throw to monkey 3

                     Monkey 1:
                       Starting items: 54, 65, 75, 74
                       Operation: new = old + 6
                       Test: divisible by 18
                         If true: throw to monkey 2
                         If false: throw to monkey 0

                     Monkey 2:
                       Starting items: 79, 60, 97
                       Operation: new = old * old
                       Test: divisible by 14
                         If true: throw to monkey 1
                         If false: throw to monkey 3

                     Monkey 3:
                       Starting items: 74
                       Operation: new = old + 3
                       Test: divisible by 18
                         If true: throw to monkey 0
                         If false: throw to monkey 1";
        let parsed = parse_input(input);

        // this check is very sensitive to larger round counts. it can overflow even with moderate
        // counts.
        let lcm = parsed.iter()
            .map(|monkey| monkey.divisible_test)
            .reduce(num::integer::lcm)
            .unwrap();
        let f1 = |w| w;
        let f2 = |w| w % lcm;
        let equiv = check_relief_equivalence(&parsed, 6, f1, f2);
        assert!(equiv);
    }
}
