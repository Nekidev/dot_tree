mod tree;
use tree::Tree;

fn main() {
    let tree = Tree::open("./tree.vpt");
    println!("{:?}", tree);
}
