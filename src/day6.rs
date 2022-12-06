
const MAX_MARKER_LEN: usize = 14;

type Marker = heapless::Vec<u8, MAX_MARKER_LEN>;

fn is_marker(window: &[u8]) -> bool {
    let mut marker = Marker::try_from(window).unwrap();
    marker.sort_unstable();
    marker.windows(2).all(|w| w[0] < w[1])
}

fn find_marker_end(input: &str, marker_len: usize) -> usize {
    if !input.is_ascii() {
        panic!("Can only do ASCII, sorry.");
    }
    let marker_start = input.as_bytes()
        .windows(marker_len)
        .position(is_marker)
        .expect("No marker found");
    marker_start + marker_len
}

fn find_start_of_packet(input: &str) -> usize {
    find_marker_end(input, 4)
}

fn find_start_of_message(input: &str) -> usize {
    find_marker_end(input, 14)
}


static INPUT: &str = include_str!("inputs/day6.txt");

pub fn run() {
    let start_of_packet = find_start_of_packet(INPUT);
    println!("First start-of-packet marker ends at offset: {start_of_packet}");

    let start_of_message = find_start_of_message(INPUT);
    println!("First start-of-message marker ends at offset: {start_of_message}");
}


#[cfg(test)]
mod test {
    use super::*;

    fn check(input: &str, packet: usize, message: usize) {
        assert_eq!(find_start_of_packet(input), packet);
        assert_eq!(find_start_of_message(input), message);
    }

    #[test]
    fn examples() {
        check("mjqjpqmgbljsphdztnvjfqwrcgsmlb", 7, 19);
        check("bvwbjplbgvbhsrlpgdmjqwftvncz", 5, 23);
        check("nppdvjthqldpwncqszvftbrmjlhg", 6, 23);
        check("nznrnfrfntjfmvfwmzdfjlvtqnbhcprsg", 10, 29);
        check("zcfzfwzzqfrljwzlrfnpqdbhtmscgvjw", 11, 26);
    }
}
