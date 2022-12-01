
//! Not really bothering with tests in this one.

static INPUT: &str = include_str!("inputs/day1.txt");

pub fn run() {
    let elves_raw = INPUT.split("\n\n").filter(|s| !s.is_empty());
    let mut elves: Vec<u32> = elves_raw.map(|elf| {
        elf.split('\n')
            .filter(|s| !s.is_empty())
            .map(|cal| cal.parse::<u32>().unwrap())
            .sum()
        })
        .collect();

    elves.sort_unstable();

    let max_single_elf = elves.last().unwrap();
    println!("Max calories carried by single elf: {max_single_elf}");

    let max_three_elves: u32 = elves.iter().rev().take(3).sum();
    println!("Total calories carried by top three elves: {max_three_elves}");
}
