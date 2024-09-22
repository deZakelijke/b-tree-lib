use std::cell::RefCell;
use std::rc::Rc;
// Perhaps T and V need some traits?
struct Node<T: PartialOrd + Clone, V: Clone> {
    keys: Vec<T>,
    values: Vec<V>,
    children: Vec<Rc<RefCell<Node<T, V>>>>,
    parent: Option<Rc<RefCell<Node<T, V>>>>,
    // max_keys: i32,
}
/// BTree
///
pub struct BTree<T: PartialOrd + Clone, V: Clone> {
    max_keys_per_node: usize,
    root: Rc<RefCell<Node<T, V>>>,
}

impl<T: PartialOrd + Clone, V: Clone> BTree<T, V> {
    pub fn new(first_key: T, first_value: V, max_keys_per_node: usize) -> Self {
        if max_keys_per_node < 4 {
            panic!("max_keys_per_node must be at least 4");
        }
        if max_keys_per_node % 2 != 0 {
            panic!("max_keys_per_node must be an even number");
        }
        let root = Rc::new(RefCell::new(Node {
            keys: vec![first_key],
            values: vec![first_value],
            children: Vec::new(),
            parent: None,
            // max_keys: max_keys_per_node,
        }));
        let tree = BTree {
            max_keys_per_node,
            root,
        };
        tree
    }

    // Returns an Err when the key already exists
    pub fn insert(&mut self, key: T, value: V) {
        let len = self.root.borrow_mut().keys.len();
        if len <= self.max_keys_per_node / 2 - 1 {
            let _ =
                BTree::insert_key_in_node(self.root.clone(), key, value, self.max_keys_per_node);
            return;
        }
        let _ = BTree::traverse_insert(self.root.clone(), key, value, self.max_keys_per_node);
    }

    // Returns an Err when the key does not exist
    pub fn remove(&mut self, _key: T) -> Result<V, ()> {
        Err(())
    }

    pub fn exists(&self, key: T) -> bool {
        // TODO: traverse_search can return an Err
        let does_exist = BTree::traverse_search(self.root.clone(), key);
        match does_exist {
            Ok(..) => return true,
            Err(..) => return false,
        };
    }

    pub fn get(&self, key: T) -> Option<V> {
        let does_exist = BTree::traverse_search(self.root.clone(), key);
        match does_exist {
            Ok(value) => return Some(value),
            Err(..) => return None,
        };
    }

    /// Traverse over the children of a node to find the node in which to insert
    ///
    /// Tries to recursively find the leaf node of the tree in which to insert
    /// the key-value pair. If the `current_node` is a leaf node, try to insert
    /// into this node. If the `current_node` has children, loop over keys of
    /// the node to find the child node to which to recurse.
    ///
    /// If the insertion into the current node fails, the error is passed to
    /// the parent node.
    /// If the recursive `traverse_insert` call on a child node fails, the
    /// child node needs to be split into two nodes, both of which become children
    /// of the `current_node`. If then the `current_node` is full, the
    /// `traverse_insert` call returns an error to the parent, which then
    /// recursively splits nodes.
    fn traverse_insert(
        current_node: Rc<RefCell<Node<T, V>>>,
        key: T,
        value: V,
        max_keys_per_node: usize,
    ) -> Result<(), &'static str> {
        if current_node.borrow().children.len() == 0 {
            // insert key in current node
            let result =
                BTree::insert_key_in_node(current_node.clone(), key, value, max_keys_per_node);
            match result {
                Ok(_) => return Ok(()),
                Err(e) => return Err(e), // Node is full, pass error to parent.
            }
        }
        let borrowed_node = current_node.borrow();
        for (i, current_key) in borrowed_node.keys.iter().enumerate() {
            if key == *current_key {
                return Err("Key already exists");
            }
            if key < *current_key {
                let child_to_traverse = Rc::clone(&borrowed_node.children[i]);
                let result = BTree::traverse_insert(
                    child_to_traverse,
                    key.clone(),
                    value.clone(),
                    max_keys_per_node,
                );
                let child_to_traverse = Rc::clone(&borrowed_node.children[i]);
                let _ = match result {
                    Ok(_) => return Ok(()),
                    Err(e) if e == "Node is full" => BTree::split_node(
                        current_node.clone(),
                        child_to_traverse,
                        key.clone(),
                        value.clone(),
                        max_keys_per_node,
                    ),
                    Err(e) => return Err(e),
                };
            }
        }
        let child_to_traverse = Rc::clone(&borrowed_node.children.last().unwrap());
        BTree::traverse_insert(
            child_to_traverse,
            key.clone(),
            value.clone(),
            max_keys_per_node,
        )
    }

    fn split_node(
        current_node: Rc<RefCell<Node<T, V>>>,
        child_to_split: Rc<RefCell<Node<T, V>>>,
        key: T,
        value: V,
        max_keys_per_node: usize,
    ) -> Result<(), &'static str> {
        let mut new_right_node = Node {
            keys: Vec::new(),
            values: Vec::new(),
            children: Vec::new(),
            parent: Some(Rc::clone(&current_node)),
        };
        let mut borrowed_child = child_to_split.borrow_mut();
        // TODO: insert key and value into the correct node
        for i in max_keys_per_node / 2..max_keys_per_node {
            // TODO: pop instead of remove. This is inefficient.
            new_right_node.keys.push(borrowed_child.keys.remove(i));
            new_right_node.values.push(borrowed_child.values.remove(i));
            new_right_node
                .children
                .push(Rc::clone(&borrowed_child.children[i]));
        }
        borrowed_child.keys.truncate(max_keys_per_node / 2);
        borrowed_child.values.truncate(max_keys_per_node / 2);
        borrowed_child.children.truncate(max_keys_per_node / 2);

        Ok(())
    }

    fn insert_key_in_node(
        current_node: Rc<RefCell<Node<T, V>>>,
        key: T,
        value: V,
        max_keys_per_node: usize,
    ) -> Result<(), &'static str> {
        let len = current_node.borrow().keys.len();
        if len >= max_keys_per_node {
            // Node is full, need to split
            // TODO: define node full error
            Err("Node is full")
        } else {
            let mut borrowed_node = current_node.borrow_mut();
            for i in 0..len {
                if key < borrowed_node.keys[i] {
                    borrowed_node.keys.insert(i, key);
                    borrowed_node.values.insert(i, value);
                    return Ok(());
                }
            }
            borrowed_node.keys.push(key);
            borrowed_node.values.push(value);
            return Ok(());
        }
    }

    fn traverse_search(current_node: Rc<RefCell<Node<T, V>>>, key: T) -> Result<V, &'static str> {
        let borrowed_node = current_node.borrow();
        if borrowed_node.children.len() > 0 {
            BTree::iterate_over_node_with_children(current_node.clone(), key)
        } else {
            BTree::iterate_over_node_without_children(current_node.clone(), key)
        }
    }

    fn iterate_over_node_with_children<'a>(
        current_node: Rc<RefCell<Node<T, V>>>,
        key: T,
    ) -> Result<V, &'static str> {
        for (i, current_key) in current_node.borrow().keys.iter().enumerate() {
            if key == *current_key {
                return Ok(current_node.borrow().values[i].clone());
            }
            if key < *current_key {
                return BTree::traverse_search(Rc::clone(&current_node.borrow().children[i]), key);
            }
        }
        return Err("Key not found");
    }

    fn iterate_over_node_without_children<'a>(
        current_node: Rc<RefCell<Node<T, V>>>,
        key: T,
    ) -> Result<V, &'static str> {
        for (i, current_key) in current_node.borrow().keys.iter().enumerate() {
            if key == *current_key {
                return Ok(current_node.borrow().values[i].clone());
            }
        }
        return Err("Key not found");
    }
}
