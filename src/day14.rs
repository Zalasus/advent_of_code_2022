
use ndarray::{s, Array2};

type LocalCoord = usize;
type LocalPoint = cgmath::Vector2<LocalCoord>;

type GlobalCoord = isize;
type GlobalPoint = cgmath::Vector2<GlobalCoord>;

/// Element-wise minimum of a and b.
fn point_min(a: GlobalPoint, b: GlobalPoint) -> GlobalPoint {
    GlobalPoint::new(a.x.min(b.x), a.y.min(b.y))
}

/// Element-wise maximum of a and b.
fn point_max(a: GlobalPoint, b: GlobalPoint) -> GlobalPoint {
    GlobalPoint::new(a.x.max(b.x), a.y.max(b.y))
}


/// A line in the global coordinate system.
struct GlobalLine {
    start: GlobalPoint,
    end: GlobalPoint,
}

impl GlobalLine {
    fn new(start: GlobalPoint, end: GlobalPoint) -> Self {
        Self {
            start,
            end,
        }
    }

    fn max(&self) -> GlobalPoint {
        point_max(self.start, self.end)
    }

    fn min(&self) -> GlobalPoint {
        point_min(self.start, self.end)
    }

    /// Walks along this line and posts all encountered coordinates to the passed closure.
    fn walk<F>(&self, mut f: F)
    where
        F: FnMut(GlobalPoint) -> (),
    {
        let step = if self.start.x == self.end.x {
            GlobalPoint::new(0, 1)
        } else if self.start.y == self.end.y {
            GlobalPoint::new(1, 0)
        } else {
            panic!("Can only walk straight lines");
        };

        let mut current = self.min();
        let end = self.max() + step; // end is inclusive for this application
        while current != end {
            f(current);
            current += step;
        }
    }
}


/// Parses a single line of input, representing a continous path of walls, into an iterator over
/// it's segments.
fn path_segments(input: &str) -> impl Iterator<Item = GlobalLine> + '_ {
    let mut points = input.split("->")
        .map(|p| {
            let (x_str, y_str) = p.trim().split_once(',').unwrap();
            let x = x_str.parse::<GlobalCoord>().unwrap();
            let y = y_str.parse::<GlobalCoord>().unwrap();
            GlobalPoint::new(x, y)
        })
        .peekable();
    std::iter::from_fn(move || Some(GlobalLine::new(points.next()?, *points.peek()?)))
}


#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
enum Tile {
    Air,
    Rock,
    Sand,
}

impl Tile {
    fn is_solid(&self) -> bool {
        match self {
            Self::Air => false,
            Self::Rock | Self::Sand => true,
        }
    }

    /// Returns a character representing this tile in a visualization.
    fn as_char(&self) -> char {
        match self {
            Self::Air => ' ',
            Self::Rock => '█',
            Self::Sand => 'o',
        }
    }
}


#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum StepResult {
    CameToRest(GlobalPoint),
    SourceBlocked,
    FellIntoVoid,
}


#[derive(Debug, Clone)]
struct Map {
    origin: GlobalPoint,
    tiles: Array2<Tile>,
    has_floor: bool,
}

impl Map {
    /// Fixed sand source, as per puzzle description.
    const SAND_SOURCE: GlobalPoint = GlobalPoint::new(500, 0);

    /// Factor by which to grow map if sand falls outside of simulation area.
    const GROWTH_STEP: LocalCoord = 1;
    const GROWTH_STEP_LEFT: GlobalCoord = -(Self::GROWTH_STEP as GlobalCoord);
    const GROWTH_STEP_RIGHT: GlobalCoord = Self::GROWTH_STEP as GlobalCoord;

    fn parse(input: &str) -> Self {
        // iterator over iterator of wall segments. ideally, we'd flatten that here, but for some
        // reason, Flatten is not Clone.
        let walls = input.lines()
            .filter(|line| !line.is_empty())
            .map(|line| path_segments(line.trim()));

        // internally, we use a local coordinate system in which the top-left of the map is always
        // at (0,0) and no negative coordinates can occur. to transform the global coordinates, we
        // need to find the minimum and maximum global coordinates.
        let map_max = walls.clone()
            .flatten()
            .fold(Self::SAND_SOURCE, |accum, line| point_max(accum, line.max()));

        let origin = walls.clone()
            .flatten()
            .fold(Self::SAND_SOURCE, |accum, line| point_min(accum, line.min()));

        // y dimension is one bigger than needed since for part two, the implicit infinite floor is
        //  two units below the lowest wall and we define the floor as just out of y bounds.
        let map_dims = (map_max - origin).cast::<LocalCoord>().unwrap()
            + LocalPoint::new(1, 2);

        let mut tiles = Array2::from_elem((map_dims.y, map_dims.x), Tile::Air);

        // draw walls
        for segment in walls.flatten() {
            segment.walk(|point| {
                let local_point = (point - origin).cast::<LocalCoord>().unwrap();
                tiles[[local_point.y, local_point.x]] = Tile::Rock;
            });
        }

        Self {
            origin,
            tiles,
            has_floor: false,
        }
    }

    fn local_sand_source(&self) -> LocalPoint {
        (Self::SAND_SOURCE - self.origin).cast::<LocalCoord>().unwrap()
    }

    fn set_has_floor(&mut self, floor: bool) {
        self.has_floor = floor;
    }

    /// Grows the simulated area to the left or right.
    fn grow(&mut self, units: GlobalCoord) {
        if units == 0 {
            return;
        }

        // create new, grown array
        let old_cols = self.tiles.ncols();
        let units_abs = units.abs() as usize;
        let new_cols = old_cols + units_abs;
        let mut new_tiles = Array2::from_elem((self.tiles.nrows(), new_cols), Tile::Air);

        // copy old array into new one
        let old_area_sliceinfo = if units < 0 {
           s![.., units_abs..]
        } else {
           s![.., ..old_cols]
        };
        new_tiles.slice_mut(old_area_sliceinfo).assign(&self.tiles);

        // if we grow to the left, our coordinate system changes
        if units < 0 {
            self.origin.x += units;
        }

        self.tiles = new_tiles;
    }

    /// Runs the simulation, attempting to place a single unit of sand at it's resting place.
    ///
    /// Oh boy, part 2 made this into a nice italian pasta dish.
    fn step(&mut self) -> StepResult {

        let source = self.local_sand_source();
        if self.tiles[[source.y, source.x]].is_solid() {
            return StepResult::SourceBlocked;
        }

        let mut sand = source;

        let one_above_floor = self.tiles.nrows() - 1;

        loop {
            // slice the map so we only look at the tiles directly below the sand
            let vert_slice = self.tiles.slice(s![(sand.y + 1).., sand.x]);
            if let Some(fall_distance) = vert_slice.iter().position(|tile| tile.is_solid()) {
                sand.y += fall_distance;
            } else {
                // no explicit solid under sand. sand falls into void or hits the infinite floor
                if self.has_floor {
                    sand.y = one_above_floor;
                    self.tiles[[sand.y, sand.x]] = Tile::Sand;
                    let global_sand = sand.cast::<GlobalCoord>().unwrap() + self.origin;
                    return StepResult::CameToRest(global_sand);
                } else {
                    return StepResult::FellIntoVoid;
                }
            }

            // sand hit something hard that is not the infinite floor. may move one down and left
            // or right if that spot is not solid.
            if sand.x == 0 {
                // at left border
                if self.has_floor {
                    // sand will fall onto floor just outside the left bounds. need to grow.
                    self.grow(Self::GROWTH_STEP_LEFT);
                    sand.x = sand.x + Self::GROWTH_STEP - 1;
                    sand.y = one_above_floor;
                } else {
                    // sand will unconditionally fall into the void out of the left bounds
                    return StepResult::FellIntoVoid;
                }
            } else if !self.tiles[[sand.y + 1, sand.x - 1]].is_solid() {
                // can move into left space
                sand.x -= 1;
                sand.y += 1;
                continue;
            } else if sand.x + 1 >= self.tiles.ncols() {
                // at right border
                if self.has_floor {
                    // sand will fall onto floor just outside the right bounds. need to grow.
                    self.grow(Self::GROWTH_STEP_RIGHT);
                    sand.x += 1;
                    sand.y = one_above_floor;
                } else {
                    // sand will unconditionally fall into the void out of the right bounds
                    return StepResult::FellIntoVoid;
                }
            } else if !self.tiles[[sand.y + 1, sand.x + 1]].is_solid() {
                // can move into right space
                sand.x += 1;
                sand.y += 1;
                continue;
            }

            // if we end up here, sand comes to rest.
            self.tiles[[sand.y, sand.x]] = Tile::Sand;
            let global_sand = sand.cast::<GlobalCoord>().unwrap() + self.origin;
            return StepResult::CameToRest(global_sand);
        }
    }
}

impl std::fmt::Display for Map {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let source = self.local_sand_source();
        let mut line = String::with_capacity(self.tiles.ncols());
        for (row_index, row) in self.tiles.outer_iter().enumerate() {
            line.clear();
            for (col_index, tile) in row.iter().enumerate() {
                if LocalPoint::new(col_index, row_index) == source && *tile == Tile::Air {
                    line.push('+');
                } else {
                    line.push(tile.as_char());
                }
            }
            writeln!(f, "{line}")?;
        }
        if self.has_floor {
            line.clear();
            for _ in 0..self.tiles.ncols() {
                line.push(Tile::Rock.as_char());
            }
            writeln!(f, "{line}")?;
        }
        Ok(())
    }
}


fn count_sand_units(mut map: Map) -> usize {
    let mut sand_units_placed = 0;
    //println!("========================");
    //println!("Initial state");
    //println!("{}", &map);
    loop {
        let result = map.step();
        //println!("------------------------");
        //println!("Result: {result:?}");
        //println!("{}", &map);
        if let StepResult::CameToRest(_) = result {
            sand_units_placed += 1;
        } else {
            break;
        }
    }
    sand_units_placed
}




static INPUT: &str = include_str!("inputs/day14.txt");

pub fn run() {
    let mut map = Map::parse(INPUT);
    let part1 = count_sand_units(map.clone());
    println!("Sand units that came to rest: {part1}");

    map.set_has_floor(true);
    let part2 = count_sand_units(map);
    println!("Sand units that came to rest with infinite floor: {part2}");
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn example() {
        let input = "498,4 -> 498,6 -> 496,6
                     503,4 -> 502,4 -> 502,9 -> 494,9";
        let mut map = Map::parse(input);
        assert_eq!(map.origin, GlobalPoint::new(494, 0));

        use StepResult::*;
        assert_eq!(map.step(), CameToRest(GlobalPoint::new(500, 8)));
        assert_eq!(map.step(), CameToRest(GlobalPoint::new(499, 8)));
        assert_eq!(map.step(), CameToRest(GlobalPoint::new(501, 8)));
        assert_eq!(map.step(), CameToRest(GlobalPoint::new(500, 7)));
        assert_eq!(map.step(), CameToRest(GlobalPoint::new(498, 8)));

        let map = Map::parse(input);
        assert_eq!(count_sand_units(map), 24);

        let mut map = Map::parse(input);
        map.set_has_floor(true);
        assert_eq!(count_sand_units(map), 93);
    }
}
