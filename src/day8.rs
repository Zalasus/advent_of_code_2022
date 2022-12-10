
use ndarray::{s, Array2, ArrayView1, ArrayViewMut1, Axis};


const ROW_AXIS: Axis = Axis(0);
const COL_AXIS: Axis = Axis(1);


fn parse_input(input: &str) -> Array2<u8> {
    let lines = input.lines().map(|line| line.trim());
    let columns = lines.clone().next().unwrap().len();
    let rows = lines.clone().count();
    let mut tree_map = Array2::from_elem((rows, columns), 0u8);
    for (y, line) in lines.enumerate() {
        for (x, tree) in line.chars().enumerate() {
            let tree_height = tree.to_digit(10).expect("Not a decimal digit");
            tree_map[[y, x]] = tree_height as u8;
        }
    }
    tree_map
}

/// Calculates the visibility for each tree in the given array when viewed along it's axis.
fn calc_visibility(input: ArrayView1<u8>, mut output: ArrayViewMut1<bool>) {
    let mut max_opt = None;
    for (vis_out, &tree) in output.iter_mut().zip(input.iter()) {
        let visible = match max_opt {
            None => {
                max_opt = Some(tree);
                true
            },
            Some(max) if tree > max => {
                max_opt = Some(tree);
                true
            },
            Some(max) if tree <= max => false,
            _ => unreachable!(),
        };
        if visible {
            *vis_out = true;
        }
    }
}

fn calc_visibility_bidirectional(input: ArrayView1<u8>, mut output: ArrayViewMut1<bool>) {
    calc_visibility(input, output.slice_mut(s![..]));
    calc_visibility(input.slice(s![..;-1]), output.slice_mut(s![..;-1]));
}

fn calc_visibility_map(tree_map: &Array2<u8>) -> Array2<bool> {
    let mut vis_map = Array2::from_elem(tree_map.raw_dim(), false);

    let input_rows = tree_map.axis_iter(ROW_AXIS);
    let output_rows = vis_map.axis_iter_mut(ROW_AXIS);
    for (input_row, output_row) in input_rows.zip(output_rows) {
        calc_visibility_bidirectional(input_row, output_row);
    }

    let input_cols = tree_map.axis_iter(COL_AXIS);
    let output_cols = vis_map.axis_iter_mut(COL_AXIS);
    for (input_col, output_col) in input_cols.zip(output_cols) {
        calc_visibility_bidirectional(input_col, output_col);
    }

    vis_map
}


fn count_visible_trees(input: ArrayView1<u8>, treehouse: u8) -> usize {
    let mut count = 0;
    for &tree in input.iter() {
        count += 1;
        if tree >= treehouse {
            break;
        }
    }
    count
}

fn calc_scenic_score_at(input: &Array2<u8>, x: usize, y: usize) -> usize {
    let treehouse = input[[y, x]];
    let left = count_visible_trees(input.slice(s![y, ..x;-1]), treehouse);
    let right = count_visible_trees(input.slice(s![y, (x+1)..]), treehouse);
    let up = count_visible_trees(input.slice(s![..y;-1, x]), treehouse);
    let down = count_visible_trees(input.slice(s![(y+1).., x]), treehouse);
    left * right * up * down
}

fn calc_scenic_score_map(input: &Array2<u8>) -> Array2<usize> {
    Array2::from_shape_fn(input.raw_dim(), |(y, x)| calc_scenic_score_at(input, x, y))
}


static INPUT: &str = include_str!("inputs/day8.txt");

pub fn run() {
    let input = parse_input(INPUT);
    let vis_map = calc_visibility_map(&input);
    let visible_trees = vis_map.iter().filter(|v| **v).count();
    println!("Trees visible from outer edge: {visible_trees}");

    let score_map = calc_scenic_score_map(&input);
    let max_score = *score_map.iter().max().unwrap();
    println!("Maximum scenic score possible: {max_score}");
}


#[cfg(test)]
mod test {
    use super::*;

    use ndarray::arr2;

    #[test]
    fn example() {
        let input = "30373
                     25512
                     65332
                     33549
                     35390";
        let expected = arr2(&[
            [3, 0, 3, 7, 3],
            [2, 5, 5, 1, 2],
            [6, 5, 3, 3, 2],
            [3, 3, 5, 4, 9],
            [3, 5, 3, 9, 0]]);
        let map = parse_input(input);
        assert_eq!(map, expected);

        let expected = arr2(&[
            [true,  true,  true,  true, true],
            [true,  true,  true, false, true],
            [true,  true, false,  true, true],
            [true, false,  true, false, true],
            [true,  true,  true,  true, true]]);
        let vis_map = calc_visibility_map(&map);
        assert_eq!(vis_map, expected);

        let score_map = calc_scenic_score_map(&map);
        assert_eq!(score_map[[1, 2]], 4);
        assert_eq!(score_map[[3, 2]], 8);
        let max_score = *score_map.iter().max().unwrap();
        assert_eq!(max_score, 8);
    }
}
