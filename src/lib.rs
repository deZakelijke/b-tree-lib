use crate::b_tree::BTree;
mod b_tree;

pub fn run() -> Result<(), ()> {
    let first_key = 0;
    let first_value = 0;
    let max_keys_per_node = 4;
    let mut b_tree = BTree::new(first_key, first_value, max_keys_per_node);

    for i in 1..5 {
        let val = i * 5;
        let result = b_tree.insert(i, val);
        println!("Key {i} inserted {val}: {result:?}");
    }
    for i in 0..5 {
        let exists = b_tree.exists(i);
        println!("Key {i} exists: {exists}");
    }
    println!("BTree: {b_tree:#?}");
    // for i in 0..4 {
    //     let value = b_tree.get(i);
    //     println!("Value for key {i} is: {value:?}");
    // }
    // for i in 0..4 {
    //     let remove = b_tree.remove(i);
    //     println!("Removed key {i}: {remove:?}");
    // }
    Err(())
}
