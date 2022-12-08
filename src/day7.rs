
use petgraph::Direction;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::DfsPostOrder;


#[derive(Debug)]
enum File<'a> {
    Regular{ name: &'a str, size: usize },
    Directory{ name: &'a str, recursive_size: Option<usize> },
}

impl File<'_> {
    fn name(&self) -> &str {
        match self {
            File::Regular{ name, .. } => name,
            File::Directory{ name, .. } => name,
        }
    }

    fn recursive_size(&self) -> Option<usize> {
        match self {
            File::Regular{ size, .. } => Some(*size),
            File::Directory{ recursive_size, .. } => *recursive_size,
        }
    }
}


#[derive(Debug, PartialEq, Eq)]
enum FsError {
    NotFound,
    NotADirectory,
    AscendedPastRoot,
    BadFileSize,
    BadCommandLine,
}


/// A wrapper around a directed graph that tracks root and present working directory (pwd), and
/// helps with constructing the file system from individual nodes.
struct FileSystem<'a> {
    tree: DiGraph<File<'a>, ()>,
    root: NodeIndex,
    pwd: NodeIndex,
}

impl<'a> FileSystem<'a> {
    fn new() -> Self {
        let mut tree = DiGraph::new();
        let root = tree.add_node(File::Directory{ name: "/", recursive_size: None });
        Self {
            tree,
            root,
            pwd: root,
        }
    }

    fn go_to_root(&mut self) {
        self.pwd = self.root;
    }

    fn descend(&mut self, dirname: &str) -> Result<(), FsError> {
        let tree = &self.tree;
        let child = tree.neighbors_directed(self.pwd, Direction::Outgoing)
            .find_map(|child_id| {
                let child_node = &tree[child_id];
                (child_node.name() == dirname).then_some((child_node, child_id))
            });
        match child {
            None => return Err(FsError::NotFound),
            Some((File::Regular{..}, _)) => return Err(FsError::NotADirectory),
            Some((File::Directory{..}, id)) => self.pwd = id,
        }
        Ok(())
    }

    fn ascend(&mut self) -> Result<(), FsError> {
        self.pwd = self.tree.neighbors_directed(self.pwd, Direction::Incoming)
            .next()
            .ok_or(FsError::AscendedPastRoot)?;
        Ok(())
    }

    fn create_file(&mut self, file: File<'a>) {
        let new_node_id = self.tree.add_node(file);
        self.tree.add_edge(self.pwd, new_node_id, ());
    }

    fn update_dir_sizes(&mut self) {
        // reset recursive size on all nodes first
        for node in self.tree.node_weights_mut() {
            match node {
                File::Regular{ .. } => (),
                File::Directory{ recursive_size, .. } => *recursive_size = None,
            }
        }

        let mut dfs = DfsPostOrder::new(&self.tree, self.root);
        while let Some(node_id) = dfs.next(&self.tree) {
            let node = &self.tree[node_id];
            let size = node.recursive_size().expect("Child nodes not already visited");

            let mut parent_node_ids = self.tree.neighbors_directed(node_id, Direction::Incoming);
            if let Some(parent_node_id) = parent_node_ids.next() {
                let parent_node = &mut self.tree[parent_node_id];
                match parent_node {
                    File::Regular{ .. } => panic!("Parent of file is a regular file"),
                    File::Directory{ recursive_size, .. } =>
                        *recursive_size.get_or_insert(0) += size
                }
            }
        }
    }

    /// Calculates the sum over the sizes of all directories less or equal in size to 100000.
    fn calc_part1(&self) -> usize {
        self.tree.node_weights()
            .filter_map(|node| match node {
                File::Directory{ recursive_size: Some(size), .. } if *size <= 100000 => Some(*size),
                _ => None,
            })
            .sum()
    }

    fn calc_part2(&self) -> usize {
        let disk_size = 70000000;
        let free_space_needed = 30000000;
        let root_size = self.tree[self.root].recursive_size().unwrap();
        let free_space_available = disk_size - root_size;
        let to_free = free_space_needed - free_space_available;
        self.tree.node_weights()
            .filter_map(|node| match node {
                File::Directory{ recursive_size: Some(size), .. } if *size > to_free => Some(*size),
                _ => None,
            })
            .min()
            .unwrap()
    }
}

fn parse_input(input: &str) -> Result<FileSystem<'_>, FsError> {
    let mut fs = FileSystem::new();
    let mut ls_mode = false;
    for line in input.lines().map(|line| line.trim()) {
        if let Some(cd) = line.strip_prefix("$ cd ") {
            ls_mode = false;
            if cd == ".." {
                fs.ascend()?;
            } else if cd == "/" {
                fs.go_to_root();
            } else {
                fs.descend(cd)?;
            }
        } else if line == "$ ls" {
            ls_mode = true;
        } else if ls_mode {
            // line should be ls output
            let (kind, name) = line.split_once(' ').ok_or(FsError::BadCommandLine)?;
            let file = if kind == "dir" {
                File::Directory {
                    name,
                    recursive_size: None,
                }
            } else {
                let size = kind.parse().map_err(|_| FsError::BadFileSize)?;
                File::Regular {
                    name,
                    size,
                }
            };
            fs.create_file(file);
        } else {
            return Err(FsError::BadCommandLine);
        }
    }

    fs.update_dir_sizes();

    Ok(fs)
}


static INPUT: &str = include_str!("inputs/day7.txt");

pub fn run() {
    let fs = parse_input(INPUT).unwrap();
    let part1 = fs.calc_part1();
    println!("Total size of all directories smaller or equal in size to 100000: {part1}");

    let part2 = fs.calc_part2();
    println!("Smallest directory to free 30000000: {part2}");
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn example() {
        let input = "$ cd /
            $ ls
            dir a
            14848514 b.txt
            8504156 c.dat
            dir d
            $ cd a
            $ ls
            dir e
            29116 f
            2557 g
            62596 h.lst
            $ cd e
            $ ls
            584 i
            $ cd ..
            $ cd ..
            $ cd d
            $ ls
            4060174 j
            8033020 d.log
            5626152 d.ext
            7214296 k";
        let fs = parse_input(input).unwrap();
        let root_file = &fs.tree[fs.root];
        assert_eq!(root_file.recursive_size().unwrap(), 48381165);
        assert_eq!(fs.calc_part1(), 95437);
        assert_eq!(fs.calc_part2(), 24933642);
    }
}
