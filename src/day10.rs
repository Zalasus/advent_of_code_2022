
use ndarray::Array2;

use std::str::FromStr;


#[derive(Debug, Copy, Clone)]
enum Instruction {
    Noop,
    Addx(i32),
}

#[derive(Debug)]
struct InstructionParseError;

impl FromStr for Instruction {
    type Err = InstructionParseError;
    fn from_str(s: &str) -> Result<Instruction, InstructionParseError> {
        if s == "noop" {
            Ok(Self::Noop)
        } else if let Some(operand) = s.strip_prefix("addx ") {
            Ok(Self::Addx(operand.parse().map_err(|_| InstructionParseError)?))
        } else {
            Err(InstructionParseError)
        }
    }
}


fn parse_input(input: &str) -> Vec<Instruction> {
    input.lines()
        .map(|line| Instruction::from_str(line.trim()).unwrap())
        .collect()
}


trait Screen {
    fn cycle(&mut self, cycle_number: usize, register: i32);
}

struct SignalAccumulator(i32);

impl SignalAccumulator {
    const RELEVANT_CYCLES: &[usize] = &[20, 60, 100, 140, 180, 220];
}

impl Screen for SignalAccumulator {
    fn cycle(&mut self, cycle_number: usize, register: i32) {
        if Self::RELEVANT_CYCLES.contains(&cycle_number) {
            self.0 += cycle_number as i32 * register;
        }
    }
}


impl Screen for Array2<bool> {
    fn cycle(&mut self, cycle_number: usize, register: i32) {
        let pixel_index = cycle_number - 1;
        let c = pixel_index % self.ncols();
        let r = (pixel_index / self.ncols()) % self.nrows();
        let sprite_range = (register-1)..=(register+1);
        self[[r, c]] = sprite_range.contains(&(c as i32));
    }
}


fn run_program(program: &[Instruction], screen: &mut impl Screen) {
    let mut cycle = 1;
    let mut register = 1;
    let mut pc = 0;
    let mut step = 0;
    while let Some(instruction) = program.get(pc) {
        screen.cycle(cycle, register);

        match instruction {
            Instruction::Noop => pc += 1,
            Instruction::Addx(op) => {
                if step == 0 {
                    step = 1;
                } else {
                    step = 0;
                    register += op;
                    pc += 1;
                }
            },
        }

        cycle += 1;
    }
}

fn accumulate_signals(program: &[Instruction]) -> i32 {
    let mut accum = SignalAccumulator(0);
    run_program(program, &mut accum);
    accum.0
}

fn render_screen(program: &[Instruction]) -> String {
    let mut screen = Array2::from_elem((6, 40), false);
    run_program(program, &mut screen);

    // turn into string for printing
    let mut string = String::new();
    for row in screen.outer_iter() {
        let row_chars = row.iter().map(|px| if *px {
            '█'
        } else {
            ' '
        })
        .chain(std::iter::once('\n'));
        string.extend(row_chars);
    }
    string
}


static INPUT: &str = include_str!("inputs/day10.txt");

pub fn run() {
    let input = parse_input(INPUT);
    let part1 = accumulate_signals(&input);
    println!("Signal accumulated during the specified cycles: {part1}");

    let part2 = render_screen(&input);
    println!("Screen rendered:\n{part2}");
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn example() {
        let input = "addx 15
                     addx -11
                     addx 6
                     addx -3
                     addx 5
                     addx -1
                     addx -8
                     addx 13
                     addx 4
                     noop
                     addx -1
                     addx 5
                     addx -1
                     addx 5
                     addx -1
                     addx 5
                     addx -1
                     addx 5
                     addx -1
                     addx -35
                     addx 1
                     addx 24
                     addx -19
                     addx 1
                     addx 16
                     addx -11
                     noop
                     noop
                     addx 21
                     addx -15
                     noop
                     noop
                     addx -3
                     addx 9
                     addx 1
                     addx -3
                     addx 8
                     addx 1
                     addx 5
                     noop
                     noop
                     noop
                     noop
                     noop
                     addx -36
                     noop
                     addx 1
                     addx 7
                     noop
                     noop
                     noop
                     addx 2
                     addx 6
                     noop
                     noop
                     noop
                     noop
                     noop
                     addx 1
                     noop
                     noop
                     addx 7
                     addx 1
                     noop
                     addx -13
                     addx 13
                     addx 7
                     noop
                     addx 1
                     addx -33
                     noop
                     noop
                     noop
                     addx 2
                     noop
                     noop
                     noop
                     addx 8
                     noop
                     addx -1
                     addx 2
                     addx 1
                     noop
                     addx 17
                     addx -9
                     addx 1
                     addx 1
                     addx -3
                     addx 11
                     noop
                     noop
                     addx 1
                     noop
                     addx 1
                     noop
                     noop
                     addx -13
                     addx -19
                     addx 1
                     addx 3
                     addx 26
                     addx -30
                     addx 12
                     addx -1
                     addx 3
                     addx 1
                     noop
                     noop
                     noop
                     addx -9
                     addx 18
                     addx 1
                     addx 2
                     noop
                     noop
                     addx 9
                     noop
                     noop
                     noop
                     addx -1
                     addx 2
                     addx -37
                     addx 1
                     addx 3
                     noop
                     addx 15
                     addx -21
                     addx 22
                     addx -6
                     addx 1
                     noop
                     addx 2
                     addx 1
                     noop
                     addx -10
                     noop
                     noop
                     addx 20
                     addx 1
                     addx 2
                     addx 2
                     addx -6
                     addx -11
                     noop
                     noop
                     noop";
        let prog = parse_input(input);
        let signal = accumulate_signals(&prog);
        assert_eq!(signal, 13140);
    }
}
