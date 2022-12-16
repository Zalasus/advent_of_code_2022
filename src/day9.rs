
use cgmath::Zero;

use std::collections::HashMap;
use std::str::FromStr;

use strum::EnumString;


type Vector = cgmath::Vector2<i32>;


#[derive(Debug, Copy, Clone, EnumString)]
enum Direction {
    #[strum(serialize = "U")]
    Up,
    #[strum(serialize = "D")]
    Down,
    #[strum(serialize = "L")]
    Left,
    #[strum(serialize = "R")]
    Right,
}

impl Direction {
    fn delta(&self) -> Vector {
        match self {
            Self::Up => Vector::new(0, -1),
            Self::Down => Vector::new(0, 1),
            Self::Left => Vector::new(-1, 0),
            Self::Right => Vector::new(1, 0),
        }
    }
}


/// A rope of N knots.
struct Rope<const N: usize> {
    knots: [Vector; N],
}

impl<const N: usize> Rope<N> {
    fn new() -> Self {
        Self {
            knots: [Vector::zero(); N],
        }
    }

    fn step_map(tail_delta: Vector) -> Option<Vector> {
        // this is the map for X. for Y, it's just transposed.
        static STEP_MAP: [[i32; 5]; 5] = [
            [-1, -1, 0, 1, 1],
            [-1,  0, 0, 0, 1],
            [-1,  0, 0, 0, 1],
            [-1,  0, 0, 0, 1],
            [-1, -1, 0, 1, 1],
        ];

        let offset = (tail_delta + Vector::new(2, 2)).cast::<usize>().unwrap();

        let x = STEP_MAP.get(offset.y)?.get(offset.x)?;
        let y = STEP_MAP.get(offset.x)?.get(offset.y)?;

        Some(Vector::new(*x, *y))
    }

    #[allow(unused_assignments)]
    fn step(&mut self, direction: Direction) {
        let mut step_delta = direction.delta();
        self.knots[0] += step_delta;
        for head_index in 0..(N-1) {
            let tail_index = head_index + 1;
            let tail_delta = self.knots[head_index] - self.knots[tail_index];
            let tail_step = Self::step_map(tail_delta).expect("Oh no the rope broke");
            self.knots[tail_index] += tail_step;
            step_delta = tail_step;
        }
    }

    #[allow(dead_code)]
    fn head(&self) -> Vector {
        *self.knots.first().unwrap()
    }

    fn tail(&self) -> Vector {
        *self.knots.last().unwrap()
    }
}

fn parse_input(input: &str) -> Vec<(Direction, usize)> {
    input.lines().map(|line| {
        let (dir_str, count_str) = line.trim().split_once(' ').unwrap();
        let dir = Direction::from_str(dir_str).unwrap();
        let count = count_str.parse().unwrap();
        (dir, count)
    }).collect()
}

fn count_visited<const N: usize>(input: &str) -> usize {
    // no need to insert start position. first step will never move the tail
    let instructions = parse_input(input);
    let mut rope = Rope::<N>::new();
    let mut map = HashMap::new();
    for (dir, count) in instructions {
        for _ in 0..count {
            rope.step(dir);
            map.insert(rope.tail(), true);
        }
    }
    map.len()
}


static INPUT: &str = include_str!("inputs/day9.txt");

pub fn run() {
    let part1 = count_visited::<2>(INPUT);
    println!("Positions visited by tail on a rope of length 2: {part1}");

    let part2 = count_visited::<10>(INPUT);
    println!("Positions visited by tail on a rope of length 10: {part2}");
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn rope() {
        let mut rope = Rope::<2>::new();
        assert_eq!(rope.head(), Vector::new(0, 0));
        assert_eq!(rope.tail(), Vector::new(0, 0));

        rope.step(Direction::Right);
        assert_eq!(rope.head(), Vector::new(1, 0));
        assert_eq!(rope.tail(), Vector::new(0, 0));

        rope.step(Direction::Right);
        assert_eq!(rope.head(), Vector::new(2, 0));
        assert_eq!(rope.tail(), Vector::new(1, 0));

        rope.step(Direction::Up);
        assert_eq!(rope.head(), Vector::new(2, -1));
        assert_eq!(rope.tail(), Vector::new(1, 0));

        rope.step(Direction::Up);
        assert_eq!(rope.head(), Vector::new(2, -2));
        assert_eq!(rope.tail(), Vector::new(2, -1));

        rope.step(Direction::Down);
        assert_eq!(rope.head(), Vector::new(2, -1));
        assert_eq!(rope.tail(), Vector::new(2, -1));
    }

    #[test]
    fn example() {
        let input = "R 4
                     U 4
                     L 3
                     D 1
                     R 4
                     D 1
                     L 5
                     R 2";
        let count = count_visited::<2>(input);
        assert_eq!(count, 13);

        let count = count_visited::<10>(input);
        assert_eq!(count, 1);

        let input = "R 5
                     U 8
                     L 8
                     D 3
                     R 17
                     D 10
                     L 25
                     U 20";
        let count = count_visited::<10>(input);
        assert_eq!(count, 36);
    }
}
