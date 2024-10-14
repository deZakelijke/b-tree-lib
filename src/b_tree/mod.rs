use std::cell::{RefCell, RefMut};
use std::fmt;
use std::fmt::Debug;
use std::rc::Rc;

struct Node<T, V>
where
    T: PartialOrd + Clone + Debug,
    V: Clone + Debug,
{
    keys: Vec<T>,
    values: Vec<V>,
    children: Vec<Rc<RefCell<Node<T, V>>>>,
    parent: Option<Rc<RefCell<Node<T, V>>>>,
    // max_keys: i32,
}
/// BTree
///
#[derive(Debug)]
pub struct BTree<T, V>
where
    T: PartialOrd + Clone + Debug,
    V: Clone + Debug,
{
    max_keys_per_node: usize,
    root: Rc<RefCell<Node<T, V>>>,
}

impl<T, V> BTree<T, V>
where
    T: PartialOrd + Clone + Debug,
    V: Clone + Debug,
{
    pub fn new(first_key: T, first_value: V, max_keys_per_node: usize) -> Self {
        if max_keys_per_node < 4 {
            // TODO: change to error
            panic!("max_keys_per_node must be at least 4");
        }
        if max_keys_per_node % 2 != 0 {
            // TODO: change to error
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
    pub fn insert(&mut self, key: T, value: V) -> Result<(), &str> {
        println!("Insertking key: {key:?}, value: {value:?}");
        let root_len = self.root.borrow_mut().keys.len();
        if root_len < self.max_keys_per_node && self.root.borrow().children.len() == 0 {
            let borrowed_root = self.root.borrow_mut();
            return BTree::insert_key_in_node(borrowed_root, key, value, self.max_keys_per_node);
        }
        let result = BTree::traverse_insert(
            self.root.clone(),
            key.clone(),
            value.clone(),
            self.max_keys_per_node,
        );
        match result {
            Ok(node) => {
                self.root = node;
                return Ok(());
            }
            Err(e) if e == "Node is full" => {
                // NOTE: why do we know this is the root? Should it be 'Node is full'?
                println!("Root node is full, splitting");
                // FIXME: this is not called when splitting the root later.
                let result = BTree::split_node(self.root.clone(), self.max_keys_per_node);
                match result {
                    Ok(node) => {
                        self.root = node;
                        return Ok(());
                    }
                    Err(e) => return Err(e),
                }
            }
            Err(e) => Err(e),
        }
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
    ) -> Result<Rc<RefCell<Node<T, V>>>, &'static str> {
        println!("Traversing node: {current_node:#?}");

        // Only insert key in current node if it is a leaf node
        {
            let borrowed_node = current_node.borrow_mut();
            if borrowed_node.children.len() == 0 {
                // insert key in current node
                let result =
                    BTree::insert_key_in_node(borrowed_node, key, value, max_keys_per_node);
                match result {
                    Ok(_) => return Ok(current_node),
                    // TODO: split child?
                    Err(e) => {
                        println!("Inserted key but node is full");
                        return Err(e);
                    } // Node is full, pass error to parent or split child.
                }
            }
        }
        let keys = {
            let borrowed_node = current_node.borrow();
            borrowed_node.keys.clone()
        };

        for (i, current_key) in keys.iter().enumerate() {
            println!("Iterating key: {current_key:?}");
            if key == *current_key {
                return Err("Key already exists");
            }
            if key < *current_key {
                let child_to_traverse = {
                    let borrowed_node = current_node.borrow();
                    Rc::clone(&borrowed_node.children[i])
                };
                let result = BTree::traverse_insert(
                    child_to_traverse,
                    key.clone(),
                    value.clone(),
                    max_keys_per_node,
                );
                let child_to_traverse = {
                    let borrowed_node = current_node.borrow();
                    Rc::clone(&borrowed_node.children[i])
                };
                match result {
                    Ok(node) => {
                        println!("Inserted key in child node, Ok");
                        return Ok(node);
                    }
                    Err(e) if e == "Node is full" => {
                        println!("Inserted key in child node, Node is full, splitting");
                        let result = BTree::split_node(child_to_traverse, max_keys_per_node);
                        match result {
                            Ok(node) => {
                                return Ok(node);
                            }
                            // Or should we split the child here?
                            Err(e) => return Err(e),
                        };
                    }
                    // Or here?
                    Err(e) if e == "Key already exists" => {
                        println!("Key already exists!");
                        return Err(e);
                    }
                    Err(_) => panic!("Invalid state?"),
                };
            }
        }
        let child_to_traverse = {
            let borrowed_node = current_node.borrow();
            Rc::clone(&borrowed_node.children.last().unwrap())
        };
        let result = BTree::traverse_insert(
            child_to_traverse,
            key.clone(),
            value.clone(),
            max_keys_per_node,
        );
        let child_to_traverse = {
            let borrowed_node = current_node.borrow();
            Rc::clone(&borrowed_node.children.last().unwrap())
        };
        match result {
            Ok(node) => {
                println!("Inserted key in child node, Ok");
                return Ok(node);
            }
            Err(e) if e == "Node is full" => {
                println!("Inserted key in child node after loop, Node is full, splitting");
                let result = BTree::split_node(child_to_traverse, max_keys_per_node);
                match result {
                    Ok(node) => {
                        return Ok(node);
                    }
                    // Or should we split the child here?
                    Err(e) => return Err(e),
                };
            }
            // Or here?
            Err(e) if e == "Key already exists" => {
                println!("Key already exists!");
                return Err(e);
            }
            Err(_) => panic!("Invalid state?"),
        }
    }

    fn split_node(
        child_to_split: Rc<RefCell<Node<T, V>>>,
        max_keys_per_node: usize,
    ) -> Result<Rc<RefCell<Node<T, V>>>, &'static str> {
        // If we split a node, do we always create two nodes?
        // When we split the root, we need to create a new root node and split the old root node.
        //
        // We do need to create a new parent when we split the root. Return the parent?
        // If the `current_node` is not None, we insert the remainder values in it.
        // Connecting the new children to the parent requres some logic
        //
        // The step of inserting into the parent, if it already existed, could require a recursive
        // call to `split_node`.
        //
        // The tree only gets taller when we split the root!
        let parent_exists;
        let parent = match child_to_split.borrow().parent.clone() {
            Some(node) => {
                parent_exists = true;
                Rc::clone(&node)
            }
            None => {
                parent_exists = false;
                Rc::new(RefCell::new(Node {
                    keys: Vec::new(),
                    values: Vec::new(),
                    children: Vec::new(),
                    parent: None,
                }))
            }
        };

        let mut new_right_node = Node {
            keys: Vec::new(),
            values: Vec::new(),
            children: Vec::new(),
            parent: Some(Rc::clone(&parent)),
        };
        let spare_key;
        let spare_value;
        {
            let mut borrowed_child = child_to_split.borrow_mut();
            borrowed_child.parent = Some(Rc::clone(&parent));
            println!("Borrowed child keys: {0:?}", borrowed_child.keys);
            println!(
                "Borrowed child children: {0:?}",
                borrowed_child.children.len()
            );
            // TODO: insert key and value into the correct node
            for i in max_keys_per_node / 2 + 1..max_keys_per_node + 1 {
                // TODO: pop instead of remove. This is inefficient.
                println!("Moving key and value with index {i} to new right node");
                new_right_node
                    .keys
                    .push(borrowed_child.keys.remove(max_keys_per_node / 2 + 1));
                new_right_node
                    .values
                    .push(borrowed_child.values.remove(max_keys_per_node / 2 + 1));
                if borrowed_child.children.len() > max_keys_per_node / 2 {
                    // if borrowed_child.children.len() > 0 ?? {
                    println!("Moving child with index {i} to new right node");
                    new_right_node
                        .children
                        .push(Rc::clone(&borrowed_child.children[i]));
                    // TODO: needs to be called +1 times since the children vec is longer than
                    // the keys vec.
                }
            }
            if borrowed_child.children.len() > max_keys_per_node / 2 {
                // if borrowed_child.children.len() > 0 ?? {
                println!("Moving child with index -1 to new right node");
                new_right_node
                    .children
                    .push(Rc::clone(&borrowed_child.children.last().unwrap()));
                // TODO: needs to be called +1 times since the children vec is longer than
                // the keys vec.
            }
            spare_key = borrowed_child.keys.remove(max_keys_per_node / 2);
            spare_value = borrowed_child.values.remove(max_keys_per_node / 2);
            println!("Spare key: {spare_key:?}, Spare value: {spare_value:?}");
            // FIXME: Should be a different value, truncating too many children
            // The +2 is a temporary fix
            let truncate_size = borrowed_child.keys.len() / 2;
            // borrowed_child.keys.truncate(truncate_size);
            // borrowed_child.values.truncate(truncate_size);
            borrowed_child.children.truncate(truncate_size + 2);
            println!("Borrowed (left) child keys: {0:?}", borrowed_child.keys);
            println!("New right node keys: {0:?}", new_right_node.keys);
        }

        let result = {
            let borrowed_parent = parent.borrow_mut();
            BTree::insert_key_in_node(
                borrowed_parent,
                spare_key.clone(),
                spare_value.clone(),
                max_keys_per_node,
            )
        };
        println!("Parent node before connecting children: {0:?}", parent);
        // If the parent Node already existed, only the new right node needs to be connected
        // How can we check if the parent already existed? At this stage, we have created
        // the parent if it did not exist.

        if parent_exists {
            println!("Parent already existed");
            BTree::connect_children_to_parent(
                parent.borrow_mut(),
                None,
                Rc::new(RefCell::new(new_right_node)),
            );
        } else {
            println!("Parent did not exist");
            BTree::connect_children_to_parent(
                parent.borrow_mut(),
                Some(Rc::clone(&child_to_split)),
                Rc::new(RefCell::new(new_right_node)),
            );
        }
        println!("Parent node after connecting children: {0:?}", parent);
        match result {
            Ok(_) => return Ok(parent),
            Err(e) if e == "Node is full" => {
                println!("Parent node is also full, splitting");
                return BTree::split_node(Rc::clone(&parent), max_keys_per_node);
            }
            Err(_) => panic!("Invalid state?"),
        }
    }
    fn connect_children_to_parent(
        mut parent: RefMut<Node<T, V>>,
        left_child: Option<Rc<RefCell<Node<T, V>>>>,
        right_child: Rc<RefCell<Node<T, V>>>,
    ) {
        // NOTE: can we assume both left and right have keys? I think so
        let keys = parent.keys.clone();
        if left_child.is_none() {
            let right_child_keys = &right_child.borrow().keys;
            let right_child_min_key = &right_child_keys[0];
            let right_child_max_key = &right_child_keys[right_child_keys.len() - 1];
            for (idx, key) in keys.iter().enumerate() {
                if idx == keys.len() - 1 {
                    parent.children.push(Rc::clone(&right_child));
                    return;
                }
                if right_child_min_key > key && right_child_max_key < &keys[idx + 1] {
                    parent.children.insert(idx + 1, Rc::clone(&right_child));
                    return;
                }
            }
        } else {
            let left_child = left_child.unwrap();
            let right_child_min_key = &right_child.borrow().keys[0];
            let left_child_keys = &left_child.borrow().keys;
            let left_child_max_key = &left_child_keys[left_child_keys.len() - 1];
            for (idx, key) in keys.iter().enumerate() {
                if left_child_max_key < key && key < right_child_min_key {
                    parent.children.insert(idx, Rc::clone(&left_child));
                    parent.children.insert(idx + 1, Rc::clone(&right_child));
                    return;
                }
            }
        }
    }

    // NOTE: move to Node module?
    fn insert_key_in_node(
        mut current_node: RefMut<Node<T, V>>,
        key: T,
        value: V,
        max_keys_per_node: usize,
    ) -> Result<(), &'static str> {
        let len = current_node.keys.len();
        for i in 0..len {
            if key < current_node.keys[i] {
                current_node.keys.insert(i, key);
                current_node.values.insert(i, value);
                return Ok(());
            }
        }
        current_node.keys.push(key);
        current_node.values.push(value);

        let len = current_node.keys.len();
        if len >= max_keys_per_node + 1 {
            return Err("Node is full");
        }
        return Ok(());
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
        return BTree::traverse_search(
            Rc::clone(&current_node.borrow().children[current_node.borrow().children.len() - 1]),
            key,
        );
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

impl<T, V> fmt::Display for Node<T, V>
where
    T: PartialOrd + Clone + Debug,
    V: Clone + Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO: implement Display for Node
        write!(f, "Node")
    }
}
impl<T, V> fmt::Debug for Node<T, V>
where
    T: PartialOrd + Clone + Debug,
    V: Clone + Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let parent = match &self.parent {
            Some(_) => Some(()),
            None => None,
        };
        write!(
            f,
            "Node {{\nkeys: {0:#?},\nvalues: {1:#?},\nchildren: {2:#?},\nparent: {3:?}\n}}",
            self.keys, self.values, self.children, parent
        )
    }
}
