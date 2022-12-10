
pub mod common;
pub mod day1;
pub mod day2;
pub mod day3;
pub mod day4;
pub mod day5;
pub mod day6;
pub mod day7;
pub mod day8;
pub mod day9;


use clap::Parser;

use colored::Colorize;

use std::ops::RangeInclusive;


/// Zalasus' advent of code 2022 entry.
#[derive(Parser)]
struct Args {
    #[clap(short, long)]
    day: Option<usize>,
}


type AocFunction = fn() -> ();


#[derive(Debug)]
enum AocError {
    NotYetSolved,
    InvalidDay,
}


#[derive(Debug)]
struct Aoc([Option<AocFunction>; 24]);

impl Aoc {
    pub const DAY_RANGE: RangeInclusive<usize> = 1..=24;

    fn add_day(&mut self, day: usize, f: AocFunction) -> Result<(), AocError> {
        if Self::DAY_RANGE.contains(&day) {
            self.0[day - 1] = Some(f);
            Ok(())
        } else {
            Err(AocError::InvalidDay)
        }
    }

    pub fn new() -> Result<Self, AocError> {
        let mut aoc = Self([None; 24]);
        aoc.add_day(1, day1::run)?;
        aoc.add_day(2, day2::run)?;
        aoc.add_day(3, day3::run)?;
        aoc.add_day(4, day4::run)?;
        aoc.add_day(5, day5::run)?;
        aoc.add_day(6, day6::run)?;
        aoc.add_day(7, day7::run)?;
        aoc.add_day(8, day8::run)?;
        aoc.add_day(9, day9::run)?;
        Ok(aoc)
    }

    pub fn get_day(&self, day: usize) -> Result<AocFunction, AocError> {
        let day_fn = self.0.get(day.wrapping_sub(1))
            .ok_or(AocError::InvalidDay)?
            .ok_or(AocError::NotYetSolved)?;
        Ok(day_fn)
    }

    pub fn run_day(&self, day: usize) {
        match self.get_day(day) {
            Ok(day_fn) => {
                eprintln!("{} {day}", "Running day".green().bold());
                day_fn();
            },
            Err(AocError::NotYetSolved) => {
                eprintln!("{} {day} {}", "Day".red().bold(), "not yet solved".red().bold());
            },
            Err(AocError::InvalidDay) => {
                eprintln!("{} {day}", "Unknown day: ".red().bold());
            },
        }
    }

    pub fn run_all_days(&self) {
        eprintln!("{}", "Running ALL DAYS".green().bold());
        eprintln!();
        for day in Self::DAY_RANGE {
            self.run_day(day);
            eprintln!();
        }
    }
}

fn main() {
    let args = Args::parse();

    let aoc = Aoc::new().unwrap();

    if let Some(day) = args.day {
        aoc.run_day(day);
    } else {
        aoc.run_all_days();
    }
}
