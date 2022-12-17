
use ndarray::Array2;

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};


type Point = [usize; 2];


struct FourNeighborhood {
    point: Point,
    index: usize,
    rows: usize,
    cols: usize,
}

impl FourNeighborhood {
    fn new(point: Point, rows: usize, cols: usize) -> Self {
        Self {
            point,
            index: 0,
            rows,
            cols,
        }
    }
}

impl Iterator for FourNeighborhood {
    type Item = Point;

    fn next(&mut self) -> Option<Point> {
        //  0
        // 3 1
        //  2
        let r = self.point[0];
        let c = self.point[1];
        while self.index < 4 {
            let index = self.index;
            self.index += 1;
            let out_r = match index {
                1 | 3 => r,
                0 if r > 0 => r - 1,
                2 if r + 1 < self.rows => r + 1,
                _ => continue,
            };
            let out_c = match index {
                0 | 2 => c,
                3 if c > 0 => c - 1,
                1 if c + 1 < self.cols => c + 1,
                _ => continue,
            };
            return Some([out_r, out_c]);
        }
        None
    }
}




struct Map {
    height_map: Array2<u8>,
    start: Point,
    end: Point,
}

fn parse_input(input: &str) -> Map {
    let lines = input.lines().map(str::trim);
    let rows = lines.clone().count();
    let columns = lines.clone().next().unwrap().chars().count();
    let mut start = None;
    let mut end = None;
    let mut height_map = Array2::from_elem((rows, columns), 0u8);
    for (row, line) in lines.enumerate() {
        for (col, point_char) in line.chars().enumerate() {
            let point = [row, col];
            let point_height = match point_char {
                'a'..='z' => point_char,
                'S' => {
                    start = Some(point);
                    'a'
                },
                'E' => {
                    end = Some(point);
                    'z'
                },
                _ => panic!("Unknown map character {point_char}"),
            };
            height_map[point] = (point_height as u32 - 'a' as u32) as u8;
        }
    }

    Map {
        height_map,
        start: start.expect("No start point found"),
        end: end.expect("No end point found"),
    }
}


struct NodeMeta {
    /// The point by which we reached this node. Used for backtracking.
    predecessor: Option<Point>,
    /// Cost for the current best known path to this node.
    cost: usize,
    /// Whether this point is already queued. Avoids searching the queue.
    in_queue: bool,
}

impl Default for NodeMeta {
    fn default() -> Self {
        Self {
            predecessor: None,
            cost: usize::MAX,
            in_queue: false,
        }
    }
}


/// A node of the A* queue, pairing a pending point and a cost. This is necessary so the queue can
/// be implemented as a priority queue.
#[derive(Debug, PartialEq, Eq)]
struct QueueNode {
    point: Point,
    /// Estimated total cost for reaching end via this node, i.e. node cost plus heuristic.
    cost: usize,
}

impl QueueNode {
    fn new(point: Point, cost: usize) -> Self {
        Self {
            point,
            cost,
        }
    }
}

impl Ord for QueueNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // reverse ordering, because Rust's BinaryHeap is a max-heap and we want a min-heap.
        self.cost.cmp(&other.cost).reverse()
    }
}

impl PartialOrd for QueueNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}


/// An implementation of the A* algorithm.
///
/// Realized as a struct so the temporary buffers created during the search can be reused.
///
/// This is simply based off the implementation described on
/// [Wikipedia](https://en.wikipedia.org/wiki/A*_search_algorithm).
struct AStar {
    node_meta: HashMap<Point, NodeMeta>,
    queue: BinaryHeap<QueueNode>,
    path_out: Vec<Point>,
}

impl AStar {
    fn new() -> Self {
        Self {
            node_meta: HashMap::new(),
            queue: BinaryHeap::new(),
            path_out: Vec::new(),
        }
    }

    /// Backtrack node meta and update path output buffer.
    fn backtrack(&mut self, end: Point) {
        self.path_out.clear();
        self.path_out.push(end);
        let mut current = end;
        while let Some(predecessor) = self.node_meta.get(&current)
            .and_then(|meta| meta.predecessor)
        {
            self.path_out.push(predecessor);
            current = predecessor;
        }
    }

    /// Calculates the manhattan distance between a and be. The classic A* heuristic on a
    /// 4-connected grid, apparently.
    fn manhattan_distance(a: Point, b: Point) -> usize {
        a.iter()
            .zip(b.iter())
            .map(|(a, b)| if a > b {
                a - b
            } else {
                b - a
            })
            .sum()
    }

    /// Runs the A* algorithm on the map.
    ///
    /// Finds a path from start to end, including both start and end. The returned path is reversed
    /// because of algorithms.
    fn run(&mut self, map: &Array2<u8>, start: Point, end: Point) -> Option<&[Point]> {
        self.queue.clear();
        self.queue.push(QueueNode::new(start, 0));
        self.node_meta.clear();
        self.node_meta.insert(start, NodeMeta {
            predecessor: None,
            cost: 0,
            in_queue: true,
        });

        while let Some(current) = self.queue.pop() {
            if current.point == end {
                // found path. backtrack
                self.backtrack(end);
                return Some(&self.path_out);
            }

            let current_height = map[current.point];
            let current_cost = {
                let current_meta = self.node_meta.entry(current.point).or_default();
                current_meta.in_queue = false; // just popped this from the queue
                current_meta.cost
            };

            for neighbor in FourNeighborhood::new(current.point, map.nrows(), map.ncols()) {
                let neighbor_height = map[neighbor];
                if neighbor_height > current_height + 1 {
                    // may only climb up 1 unit. ignore this neighbor
                    continue;
                }

                // calculate part cost for this neighbor via current node
                let edge_cost = 1;
                let neighbor_cost = current_cost.saturating_add(edge_cost);

                let neighbor_meta = self.node_meta.entry(neighbor).or_default();
                if neighbor_cost < neighbor_meta.cost {
                    // path to neighbor via current is better than it's previous path! update it.
                    neighbor_meta.predecessor = Some(current.point);
                    neighbor_meta.cost = neighbor_cost;

                    // estimate total cost for queue priority
                    let heuristic = Self::manhattan_distance(neighbor, end);
                    let neighbor_total_cost = neighbor_cost + heuristic;

                    if !neighbor_meta.in_queue {
                        neighbor_meta.in_queue = true;
                        self.queue.push(QueueNode::new(neighbor, neighbor_total_cost));
                    }
                }
            }
        }

        // no path to target
        None
    }
}

/// Ignores the map-defined start point and instead checks all points with height 'a'.
///
/// Yeah, yeah, should've went with Dijkstra. But let's roll with A* for the lols. Oh god it's so
/// slow.
fn find_min_path_len(map: &Map) -> usize {
    let mut a_star = AStar::new();
    map.height_map.indexed_iter()
        .filter_map(|(index, height)| (*height == 0).then_some(index))
        .filter_map(|start| {
            let start = [start.0, start.1];
            a_star.run(&map.height_map, start, map.end).map(|path| path.len() - 1)
        })
        .min()
        .unwrap()
}

static INPUT: &str = include_str!("inputs/day12.txt");

pub fn run() {
    let map = parse_input(INPUT);
    let mut a_star = AStar::new();
    let path = a_star.run(&map.height_map, map.start, map.end).unwrap();
    println!("The shortest path from start to end is {} steps long", path.len() - 1);

    let min_path = find_min_path_len(&map);
    println!("Minimum path starting from an 'a' node: {min_path}");
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn example() {
        let input = "Sabqponm
                     abcryxxl
                     accszExk
                     acctuvwj
                     abdefghi";
        let parsed = parse_input(input);
        assert_eq!(parsed.start, [0, 0]);
        assert_eq!(parsed.end, [2, 5]);

        let mut a_star = AStar::new();
        let path = a_star.run(&parsed.height_map, parsed.start, parsed.end).unwrap();
        assert_eq!(path.len() - 1, 31);

        let min_path = find_min_path_len(&parsed);
        assert_eq!(min_path, 29);
    }
}
