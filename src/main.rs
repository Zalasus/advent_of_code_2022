
pub mod common;
pub mod day1;
pub mod day2;
pub mod day3;
pub mod day4;
pub mod day5;
pub mod day6;


use clap::Parser;

use colored::Colorize;


const DAYS: usize = 6;


/// Zalasus' advent of code 2022 entry.
#[derive(Parser)]
struct Args {
    #[clap(short, long)]
    day: Option<usize>,

    /// Enables interactive puzzles.
    #[clap(short, long)]
    interactive: bool,
}

fn run_day(day: usize, _args: &Args) {
    println!("\n{} {}", "Running day".green().bold(), day);
    match day {
        1 => day1::run(),
        2 => day2::run(),
        3 => day3::run(),
        4 => day4::run(),
        5 => day5::run(),
        6 => day6::run(),
        _ => eprintln!("{} {}", "Unknown day: ".red().bold(), day),
    }
}

fn main() {
    let args = Args::parse();

    if let Some(day) = args.day {
        run_day(day, &args);
    } else {
        println!("{}", "[Running all days]".green().bold());
        for day in 1..=DAYS {
            run_day(day, &args);
        }
    }
}
