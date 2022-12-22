
use std::cmp::Ordering;


#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum PacketToken {
    ListStart,
    ListEnd,
    Comma,
    Number(u64),
}

struct PacketTokenizer<'a>(&'a str);

impl<'a> PacketTokenizer<'a> {
    fn next_token(&mut self) -> Option<PacketToken> {
        self.0 = self.0.trim_start();
        let sigil = self.0.chars().next()?;
        let sigil_len = sigil.len_utf8();
        let (token, len) = match sigil {
            '[' => (PacketToken::ListStart, sigil_len),
            ']' => (PacketToken::ListEnd, sigil_len),
            ',' => (PacketToken::Comma, sigil_len),
            '0'..='9' => {
                let end = self.0.find(|c| "[],".contains(c)).expect("Number token end not found");
                let number = self.0[..end].parse().expect("Number parse error");
                (PacketToken::Number(number), end)
            },
            _ => panic!("Unrecognized character {sigil}"),
        };
        self.0 = &self.0[len..];
        Some(token)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum FlatPacketItem {
    Number(u64),
    List{ size: usize, flat_size: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Packet(Vec<FlatPacketItem>);

impl Packet {
    fn parse(s: &str) -> Self {
        let mut items = Vec::new();
        let mut tokens = PacketTokenizer(s);
        let mut list_index_stack = Vec::new();
        while let Some(token) = tokens.next_token() {
            let got_new_item = match token {
                PacketToken::Number(n) => {
                    items.push(FlatPacketItem::Number(n));
                    true
                },
                PacketToken::ListStart => {
                    // new list. add this to stack *after* all the existing lists have been updated
                    // so it does not count itself.
                    items.push(FlatPacketItem::List{ size: 0, flat_size: 0 });
                    true
                },
                PacketToken::ListEnd => {
                    list_index_stack.pop();
                    false
                },
                PacketToken::Comma => false, // commas are not really needed by this parser, except
                                             // for validation perhaps.
            };

            if got_new_item {
                // the top list (the one we are currently building gets a new item
                if let Some(top_index) = list_index_stack.last() {
                    if let FlatPacketItem::List{ size, .. } = &mut items[*top_index] {
                        *size += 1;
                    } else {
                        panic!("Not a list");
                    }
                }

                // all lists' flat size grows by one, also, since one now need to skip one more
                // element to skip over all their children.
                for index in &list_index_stack {
                    if let FlatPacketItem::List{ flat_size, .. } = &mut items[*index] {
                        *flat_size += 1;
                    } else {
                        panic!("Not a list");
                    }
                }
            }

            if token == PacketToken::ListStart {
                list_index_stack.push(items.len() - 1);
            }
        }

        Self(items)
    }

    fn slice(&self) -> PacketSlice<'_> {
        PacketSlice(&self.0[..])
    }

    #[allow(dead_code)]
    fn iter(&self) -> PacketIter<'_> {
        self.slice().iter()
    }
}

impl PartialOrd for Packet {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        Some(self.cmp(rhs))
    }
}

impl Ord for Packet {
    fn cmp(&self, rhs: &Self) -> Ordering {
        self.slice().packet_cmp(rhs.slice())
    }
}


/// A packet item. Can be either a number or a list.
#[derive(Debug, Copy, Clone)]
enum PacketItem<'a> {
    Number(u64),
    List(PacketSlice<'a>),
}


/// A slice of a packet.
#[derive(Debug, Copy, Clone)]
struct PacketSlice<'a>(&'a [FlatPacketItem]);

impl<'a> PacketSlice<'a> {
    fn iter(&self) -> PacketIter<'a> {
        PacketIter(self.0)
    }

    /// This defines a total order, which sorta corresponds to the one described in part 1.
    ///
    /// In reference to that, the packets are in "correct order" when the lhs is smaller than the
    /// rhs. The equal case has never been described, but this routine should coincide with the
    /// simpler equality case.
    fn packet_cmp(&self, rhs: Self) -> Ordering {
        use PacketItem::*;
        let mut left_iter = self.iter();
        let mut right_iter = rhs.iter();
        loop {
            let ordering = match (left_iter.next(), right_iter.next()) {
                (Some(Number(left)), Some(Number(right))) => left.cmp(&right),
                (Some(List(left_list)), Some(List(right_list)))
                    => left_list.packet_cmp(right_list),
                (Some(List(left_list)), Some(Number(right_number))) => {
                    let tmp_slice = &[FlatPacketItem::Number(right_number)];
                    left_list.packet_cmp(PacketSlice(tmp_slice))
                },
                (Some(Number(left_number)), Some(List(right_list))) => {
                    let tmp_slice = &[FlatPacketItem::Number(left_number)];
                    PacketSlice(tmp_slice).packet_cmp(right_list)
                },
                (None, None) => return Ordering::Equal,
                (Some(_), None) => return Ordering::Greater,
                (None, Some(_)) => return Ordering::Less,
            };

            match ordering {
                Ordering::Less | Ordering::Greater => return ordering,
                Ordering::Equal => (), // need to check more elements
            }
        }
    }
}


/// An iterator over the items of a packet slice.
///
/// This will not automatically descend into lists. You have to do that yourself.
struct PacketIter<'a>(&'a [FlatPacketItem]);

impl<'a> Iterator for PacketIter<'a> {
    type Item = PacketItem<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.0.first() {
            None => None,
            Some(FlatPacketItem::Number(n)) => {
                self.0 = &self.0[1..];
                Some(PacketItem::Number(*n))
            },
            Some(FlatPacketItem::List{ flat_size, .. }) => {
                let slice = PacketSlice(&self.0[1..=*flat_size]);
                self.0 = &self.0[(*flat_size + 1)..];
                Some(PacketItem::List(slice))
            },
        }
    }
}

fn is_in_order(left: &str, right: &str) -> bool {
    let left_packet = Packet::parse(left);
    let right_packet = Packet::parse(right);
    match left_packet.cmp(&right_packet) {
        Ordering::Less => true,
        Ordering::Greater => false,
        Ordering::Equal => panic!("I think this case is not even defined"),
    }
}

fn part_one(input: &str) -> usize {
    input.split("\n\n")
        .map(|pair| pair.split_once('\n').unwrap())
        .enumerate()
        .filter_map(|(index, (left, right))| is_in_order(left, right).then_some(index + 1))
        .sum()
}

fn part_two(input: &str) -> usize {
    let mut packets = input.lines()
        .filter(|line| !line.is_empty())
        .map(|line| Packet::parse(line))
        .collect::<Vec<_>>();

    let divider_1 = Packet::parse("[[2]]");
    let divider_2 = Packet::parse("[[6]]");
    packets.push(divider_1.clone());
    packets.push(divider_2.clone());
    packets.sort_unstable();

    let divider_1_pos = packets.binary_search(&divider_1).ok().unwrap();
    let divider_2_pos = packets.binary_search(&divider_2).ok().unwrap();

    (divider_1_pos + 1) * (divider_2_pos + 1)
}


static INPUT: &str = include_str!("inputs/day13.txt");

pub fn run() {
    let part1 = part_one(INPUT);
    println!("Sum of indices of packets that are in right order: {part1}");

    let part2 = part_two(INPUT);
    println!("Decoder key: {part2}");
}


#[cfg(test)]
mod test {
    use super::*;

    use assert2::{assert, let_assert};

    #[test]
    fn example() {
        let packet = Packet::parse("[[1],[2,3,4],[[]],5]");
        let expected = {
            use FlatPacketItem::*;
            Packet(vec![
                List{ size: 4, flat_size: 9 },
                List{ size: 1, flat_size: 1 },
                Number(1),
                List{ size: 3, flat_size: 3 },
                Number(2),
                Number(3),
                Number(4),
                List{ size: 1, flat_size: 1 },
                List{ size: 0, flat_size: 0 },
                Number(5),
            ])
        };
        assert!(packet == expected);

        let packet = Packet::parse("[1,[2,3]]");
        let mut iter = packet.iter();
        let_assert!(Some(PacketItem::List(sublist)) = iter.next());
        assert!(iter.next().is_none());

        let mut subiter = sublist.iter();
        assert!(let Some(PacketItem::Number(1)) = subiter.next());
        let_assert!(Some(PacketItem::List(subsublist)) = subiter.next());
        assert!(subiter.next().is_none());

        let mut subsubiter = subsublist.iter();
        assert!(let Some(PacketItem::Number(2)) = subsubiter.next());
        assert!(let Some(PacketItem::Number(3)) = subsubiter.next());
        assert!(subsubiter.next().is_none());

        assert!(is_in_order("[1,1,3,1,1]", "[1,1,5,1,1]"));
        assert!(is_in_order("[[1],[2,3,4]]", "[[1],4]"));
        assert!(is_in_order("[[4,4],4,4]", "[[4,4],4,4,4]"));
        assert!(!is_in_order("[1,[2,[3,[4,[5,6,7]]]],8,9]","[1,[2,[3,[4,[5,6,0]]]],8,9]"));
    }
}
