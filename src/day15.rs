
use itertools::Itertools;

use std::collections::HashSet;


type Coord = i64;
type Point = cgmath::Vector2<Coord>;


fn point_from_coords(s: &str) -> Point {
    let mut x = None;
    let mut y = None;
    let items = s.split(',')
        .map(|item| item.trim().split_once('=').unwrap());
    for (dim, value) in items{
        match dim {
            "x" => x = Some(value.parse().unwrap()),
            "y" => y = Some(value.parse().unwrap()),
            _ => panic!("Unknown dimension {dim}"),
        }
    }

    Point::new(x.unwrap(), y.unwrap())
}

fn manhattan_distance(a: Point, b: Point) -> Coord {
    // why does cgmatch not have this?
    let diff = a - b;
    let arr: [Coord; 2] = diff.into();
    arr.iter().map(|c| c.abs()).sum()
}

struct Sensor {
    position: Point,
    beacon: Point,
    range: Coord,
}

impl Sensor {
    fn parse(s: &str) -> Sensor {
        let (sensor, beacon) = s.trim().split_once(':').unwrap();
        let sensor_coords = sensor.strip_prefix("Sensor at ").unwrap();
        let beacon_coords = beacon.strip_prefix(" closest beacon is at ").unwrap();
        let sensor = point_from_coords(sensor_coords);
        let beacon = point_from_coords(beacon_coords);
        Self {
            position: sensor,
            beacon,
            range: manhattan_distance(sensor, beacon),
        }
    }

    fn is_in_range(&self, p: Point) -> bool {
        manhattan_distance(self.position, p) <= self.range
    }
}


struct Map {
    sensors: Vec<Sensor>,
    beacons: HashSet<Point>,
    x_min: Coord,
    x_max: Coord,
}

impl Map {
    fn parse(s: &str) -> Self {
        let sensors = s.lines()
            .map(|line| Sensor::parse(line))
            .collect::<Vec<_>>();

        let beacons = sensors.iter()
            .map(|sensor| sensor.beacon)
            .collect::<HashSet<_>>();

        let (x_min, x_max) = sensors.iter()
            .map(|sensor| {
                let x = sensor.position.x;
                let range = sensor.range;
                [x - range, x + range].into_iter()
            })
            .flatten()
            .minmax()
            .into_option()
            .unwrap();

        Self {
            sensors,
            beacons,
            x_min,
            x_max,
        }
    }

    fn count_nobeacon_cells(&self, y: Coord) -> usize {
        let mut ranges = self.sensors.iter()
            .filter_map(|s| {
                let y_diff = s.position.y.abs_diff(y);
                if y_diff <= s.range {
                    let start = s.position.x - s.range + y_diff;
                    let end = s.position.x + s.range - y_diff;
                    Some(start..=end)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        ranges.sort_unstable_by_key(|r| r.0);

        let mut joint_ranges = Vec::new();
        for window in ranges.windows(2) {
            if window[1].contains(window[0].end) {
                joint_ranges.push(window[0].start..window[1].end)
            }
        }
        ranges.windows(2)
            .map(|w| if w

        /*(self.x_min..=self.x_max).into_iter()
            .map(|x| Point::new(x, y))
            .filter(|p| !self.beacons.contains(p))
            .filter(|p| self.sensors.iter().any(|s| s.is_in_range(*p)))
            .count()
        */
    }
}


static INPUT: &str = include_str!("inputs/day15.txt");

pub fn run() {
    let map = Map::parse(INPUT);
    let row = 2000000;
    let part1 = map.count_nobeacon_cells(row);
    println!("Positions at which no beacon can be present in row {row}: {part1}");
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn example() {
        let input = "Sensor at x=2, y=18: closest beacon is at x=-2, y=15
                     Sensor at x=9, y=16: closest beacon is at x=10, y=16
                     Sensor at x=13, y=2: closest beacon is at x=15, y=3
                     Sensor at x=12, y=14: closest beacon is at x=10, y=16
                     Sensor at x=10, y=20: closest beacon is at x=10, y=16
                     Sensor at x=14, y=17: closest beacon is at x=10, y=16
                     Sensor at x=8, y=7: closest beacon is at x=2, y=10
                     Sensor at x=2, y=0: closest beacon is at x=2, y=10
                     Sensor at x=0, y=11: closest beacon is at x=2, y=10
                     Sensor at x=20, y=14: closest beacon is at x=25, y=17
                     Sensor at x=17, y=20: closest beacon is at x=21, y=22
                     Sensor at x=16, y=7: closest beacon is at x=15, y=3
                     Sensor at x=14, y=3: closest beacon is at x=15, y=3
                     Sensor at x=20, y=1: closest beacon is at x=15, y=3";
        let map = Map::parse(input);
        assert_eq!(map.sensors.len(), 14);
        assert_eq!(map.sensors[3].position, Point::new(12, 14));
        assert_eq!(map.sensors[3].beacon, Point::new(10, 16));
        assert_eq!(map.beacons.len(), 6);

        assert_eq!(map.count_nobeacon_cells(10), 26);
    }
}
