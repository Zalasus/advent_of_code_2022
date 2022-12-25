
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


#[derive(Copy, Clone)]
struct CoordRange {
    start: Coord,
    end: Coord,
}

impl CoordRange {
    fn new(start: Coord, end: Coord) -> Self {
        Self {
            start,
            end,
        }
    }

    fn overlaps(&self, other: &Self) -> bool {
        let self_range = self.start..=self.end;
        let other_range = other.start..=other.end;
        other_range.contains(&self.start) || other_range.contains(&self.end)
            || self_range.contains(&other.start)
    }

    fn try_join(&self, other: &Self) -> Option<Self> {
        if self.overlaps(other) {
            Some(Self {
                start: self.start.min(other.start),
                end: self.end.max(other.end),
            })
        } else {
            None
        }
    }

    fn len(&self) -> Coord {
        (self.end - self.start).abs()
    }
}


struct Sensor {
    position: Point,
    _beacon: Point,
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
            _beacon: beacon,
            range: manhattan_distance(sensor, beacon),
        }
    }
}


struct Map {
    sensors: Vec<Sensor>,
}

impl Map {
    fn parse(s: &str) -> Self {
        let sensors = s.lines()
            .map(|line| Sensor::parse(line))
            .collect::<Vec<_>>();

        Self {
            sensors,
        }
    }
}


struct BeaconFinder<'a> {
    map: &'a Map,
    ranges: Vec<CoordRange>,
    joint_ranges: Vec<CoordRange>,
}

impl<'a> BeaconFinder<'a> {
    fn new(map: &'a Map) -> Self {
        Self {
            map,
            ranges: Vec::with_capacity(map.sensors.len()),
            joint_ranges: Vec::with_capacity(map.sensors.len()),
        }
    }

    /// Collects the ranges of x coordinates that are covered by the sensors and joins overlapping
    /// ranges.
    fn collect_ranges(&mut self, y: Coord) {
        self.ranges.clear();
        self.joint_ranges.clear();

        // this utilized the rectangular shape of the L1 norm:
        self.ranges.extend(self.map.sensors.iter()
            .filter_map(|s| {
                let y_diff = (s.position.y - y).abs();
                if y_diff <= s.range {
                    let start = s.position.x - s.range + y_diff;
                    let end = s.position.x + s.range - y_diff;
                    Some(CoordRange::new(start, end))
                } else {
                    None
                }
            }));

        self.ranges.sort_unstable_by_key(|r| r.start);

        if let Some(current) = self.ranges.first() {
            let mut current = *current;
            for range in self.ranges.iter().skip(1) {
                if let Some(joint) = current.try_join(range) {
                    current = joint;
                } else {
                    self.joint_ranges.push(current);
                    current = *range;
                }
            }
            self.joint_ranges.push(current);
        }
    }

    fn count_nobeacon_cells(&mut self, y: Coord) -> usize {
        self.collect_ranges(y);
        self.joint_ranges.iter()
            .map(|range| range.len())
            .sum::<Coord>()
            .try_into()
            .unwrap()
    }

    fn find_beacon(&mut self, max: Coord) -> Point {
        // do the same as in part 1, but this time, look for a hole in the range of coordinates.
        //  corner cutting: this will not check whether the hole is unique.
        //  searching only the borders of sensors is probably more efficient than this, but meh.
        for y in 0..max {
            self.collect_ranges(y);

            // edge case: only one range with the hole right at the x border
            if self.joint_ranges.len() == 1 {
                let range = self.joint_ranges[0];
                if range.start == 1 {
                    return Point::new(0, y);
                } else if range.end == (max - 1) {
                    return Point::new(max, y);
                }
            }

            for window in self.joint_ranges.windows(2) {
                if window[0].end >= 0 && window[1].start <= max {
                    return Point::new(window[0].end + 1, y);
                }
            }
        }

        panic!("No hole found");
    }
}


static INPUT: &str = include_str!("inputs/day15.txt");

pub fn run() {
    let map = Map::parse(INPUT);
    let mut finder = BeaconFinder::new(&map);
    let row = 2000000;
    let part1 = finder.count_nobeacon_cells(row);
    println!("Positions at which no beacon can be present in row {row}: {part1}");

    let max = 4000000;
    let beacon = finder.find_beacon(max);
    let part2 = beacon.x * max + beacon.y;
    println!("Beacon at {beacon:?}. Frequency: {part2}");
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

        let mut finder = BeaconFinder::new(&map);
        assert_eq!(finder.count_nobeacon_cells(10), 26);
        assert_eq!(finder.find_beacon(20), Point::new(14, 11));
    }
}
