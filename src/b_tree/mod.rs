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
    pub fn insert(&mut self, key: T, value: V) -> Result<(), &str> {
        let root_len = self.root.borrow_mut().keys.len();
        if root_len < self.max_keys_per_node {
            return BTree::insert_key_in_node(
                self.root.clone(),
                key,
                value,
                self.max_keys_per_node,
            );
        }
        let result = BTree::traverse_insert(
            self.root.clone(),
            key.clone(),
            value.clone(),
            self.max_keys_per_node,
        );
        match result {
            Ok(_) => return Ok(()),
            Err(e) if e == "Node is full" => {
                // NOTE: why do we know this is the root? Should it be 'Node is full'?
                println!("Root node is full, splitting");
                let result = BTree::split_node(None, self.root.clone(), self.max_keys_per_node);
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
    ) -> Result<(), &'static str> {
        println!("Traversing node: {current_node:?}");
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
                    Err(e) if e == "Node is full" => {
                        let result = BTree::split_node(
                            Some(current_node.clone()),
                            child_to_traverse,
                            max_keys_per_node,
                        );
                        match result {
                            Ok(_) => {
                                return Ok(());
                            }
                            Err(e) => return Err(e),
                        };
                    }
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
        current_node: Option<Rc<RefCell<Node<T, V>>>>,
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
        let parent;
        match current_node {
            Some(node) => {
                parent = Rc::clone(&node);
            }
            None => {
                parent = Rc::new(RefCell::new(Node {
                    keys: Vec::new(),
                    values: Vec::new(),
                    children: Vec::new(),
                    parent: None,
                }));
            }
        }

        let mut new_right_node = Node {
            keys: Vec::new(),
            values: Vec::new(),
            children: Vec::new(),
            parent: Some(Rc::clone(&parent)),
        };
        // TODO: set parent of child_to_split
        println!("New node created: {new_right_node:?}");
        let spare_key;
        let spare_value;
        {
            let mut borrowed_child = child_to_split.borrow_mut();
            println!("Borrowed child keys: {0:?}", borrowed_child.keys);
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
                if borrowed_child.children.len() > i {
                    new_right_node.children.push(Rc::clone(
                        &borrowed_child.children[max_keys_per_node / 2 + 1],
                    ));
                }
            }
            spare_key = borrowed_child.keys.remove(max_keys_per_node / 2);
            spare_value = borrowed_child.values.remove(max_keys_per_node / 2);
            println!("Spare key: {spare_key:?}, Spare value: {spare_value:?}");
            borrowed_child.keys.truncate(max_keys_per_node / 2);
            borrowed_child.values.truncate(max_keys_per_node / 2);
            borrowed_child.children.truncate(max_keys_per_node / 2);
        }

        let result = BTree::insert_key_in_node(
            Rc::clone(&parent),
            spare_key.clone(),
            spare_value.clone(),
            max_keys_per_node,
        );
        BTree::connect_children_to_parent(
            parent.borrow_mut(),
            Rc::clone(&child_to_split),
            Rc::new(RefCell::new(new_right_node)),
        );
        match result {
            Ok(_) => return Ok(parent),
            Err(e) if e == "Node is full" => {
                let parent_ref = parent.borrow_mut();
                return BTree::split_node(
                    parent_ref.parent.clone(),
                    Rc::clone(&parent),
                    max_keys_per_node,
                );
            }
            Err(_) => panic!("Invalid state?"),
        }
    }
    fn connect_children_to_parent(
        mut parent: RefMut<Node<T, V>>,
        left_child: Rc<RefCell<Node<T, V>>>,
        right_child: Rc<RefCell<Node<T, V>>>,
    ) {
        // NOTE: can we assume both left and right have keys? I think so
        let right_child_min_key = &right_child.borrow().keys[0];
        let left_child_keys = &left_child.borrow().keys;
        let left_child_max_key = &left_child_keys[left_child_keys.len() - 1];
        let keys = parent.keys.clone();
        for (idx, key) in keys.iter().enumerate() {
            if left_child_max_key < key && key < right_child_min_key {
                parent.children.insert(idx, Rc::clone(&left_child));
                parent.children.insert(idx + 1, Rc::clone(&right_child));
            }
        }
    }

    // NOTE: move to Node module?
    fn insert_key_in_node(
        current_node: Rc<RefCell<Node<T, V>>>,
        key: T,
        value: V,
        max_keys_per_node: usize,
    ) -> Result<(), &'static str> {
        let len = current_node.borrow().keys.len();
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

        let len = borrowed_node.keys.len();
        if len >= max_keys_per_node + 1 {
            // TODO: define node full error
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
