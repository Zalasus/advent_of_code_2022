
use std::fmt::Debug;
use std::str::FromStr;

use strum::EnumString;


#[derive(Debug, Copy, Clone, PartialEq, Eq, EnumString)]
enum Shape {
    #[strum(serialize = "A", serialize = "X")]
    Rock,
    #[strum(serialize = "B", serialize = "Y")]
    Paper,
    #[strum(serialize = "C", serialize = "Z")]
    Scissors,
}

impl Shape {
    fn weak_against(&self) -> Self {
        match self {
            Self::Rock => Self::Paper, // why though?
            Self::Paper => Self::Scissors,
            Self::Scissors => Self::Rock,
        }
    }

    fn strong_against(&self) -> Self {
        match self {
            Self::Rock => Self::Scissors,
            Self::Paper => Self::Rock,
            Self::Scissors => Self::Paper,
        }
    }

    fn score(&self) -> u32 {
        match self {
            Self::Rock => 1,
            Self::Paper => 2,
            Self::Scissors => 3,
        }
    }

    fn play(&self, them: Self) -> PlayResult {
        if *self == them {
            PlayResult::Draw
        } else if self.strong_against() == them {
            PlayResult::Win
        } else {
            PlayResult::Loss
        }
    }
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, EnumString)]
enum PlayResult {
    #[strum(serialize = "Z")]
    Win,
    #[strum(serialize = "X")]
    Loss,
    #[strum(serialize = "Y")]
    Draw,
}

impl PlayResult {
    fn score(&self) -> u32 {
        match self {
            Self::Win => 6,
            Self::Loss => 0,
            Self::Draw => 3,
        }
    }

    fn solve_play(&self, them: Shape) -> Shape {
        match self {
            Self::Win => them.weak_against(),
            Self::Loss => them.strong_against(),
            Self::Draw => them,
        }
    }
}


fn parse_input<L, R>(input: &str) -> Vec<(L, R)>
where
    L: FromStr + Debug,
    R: FromStr + Debug,
{
    input.split('\n')
        .filter(|line| !line.is_empty())
        .map(|line| {
            let (l, r) = line.trim().split_once(' ').unwrap();
            (L::from_str(l).ok().unwrap(), R::from_str(r).ok().unwrap())
        })
        .collect()
}

fn calculate_score_part1(input: &str) -> u32 {
    let parsed: Vec<(Shape, Shape)> = parse_input(input);
    parsed.iter()
        .map(|(them, us)| us.score() + us.play(*them).score())
        .sum()
}

fn calculate_score_part2(input: &str) -> u32 {
    let parsed: Vec<(Shape, PlayResult)> = parse_input(input);
    parsed.iter()
        .map(|(them, result)| result.score() + result.solve_play(*them).score())
        .sum()
}

static INPUT: &str = include_str!("inputs/day2.txt");

pub fn run() {
    let part1 = calculate_score_part1(INPUT);
    println!("Score if second column is a shape: {part1}");

    let part2 = calculate_score_part2(INPUT);
    println!("Score if second column is a play result: {part2}");
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn example() {
        let input = "
            A Y
            B X
            C Z";
        let score = calculate_score_part1(input);
        assert_eq!(score, 15);

        let score = calculate_score_part2(input);
        assert_eq!(score, 12);
    }
}
