use crate::b_tree::BTree;
mod b_tree;

pub fn run() -> Result<(), ()> {
    let first_key = 1;
    let first_value = 100;
    let max_keys_per_node = 4;
    let mut b_tree = BTree::new(first_key, first_value, max_keys_per_node);

    let _ = b_tree.insert(1, 10);
    for i in 0..10 {
        let _ = b_tree.insert(i, i * 10 % 3);
    }
    for i in 0..5 {
        let exists = b_tree.exists(i);
        println!("Key {i} exists: {exists}");
    }
    for i in 5..10 {
        let value = b_tree.get(i);
        println!("Value for key {i} is: {value:?}");
    }
    for i in 0..10 {
        let _ = b_tree.remove(i);
    }
    Err(())
}
