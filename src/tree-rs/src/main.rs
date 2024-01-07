mod tree;

fn main() {
    let what = "read";

    let tree: tree::Tree;
    if what == "create" {
        tree = match tree::Tree::create(
            "./tree.tree",
            vec![],
            vec![256_u32, 32_u32]
        ) {
            Ok(t) => t,
            Err(e) => panic!("{:?}", e),
        };
    } else {
        tree = match tree::Tree::open("./tree.tree") {
            Ok(t) => t,
            Err(e) => panic!("{:?}", e),
        };
    }

    println!("{:?}", tree);
}
