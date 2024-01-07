mod tree;

fn main() {
    let mut tree: tree::Tree = match tree::Tree::open(
        "./tree.tree",
        tree::TreeOpenMode::ReadWrite
    ) {
        Ok(t) => t,
        Err(e) => match tree::Tree::create(
            "./tree.tree",
            tree::TreeOpenMode::ReadWrite,
            vec![tree::Feature::Disabling],
            vec![4_u32],
        ) {
            Ok(t) => t,
            Err(e) => panic!("{:?}", e),
        },
    };

    println!("{:?}", tree);
    dbg!(tree.nodes());
    dbg!(tree.add_node(vec![vec![true, false, false, true]], 0, true));
    dbg!(tree.get_node(0));
}
