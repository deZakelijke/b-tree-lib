use std::cell::RefCell;
use std::rc::Rc;
// Perhaps T and V need some traits?
struct Node<T: PartialOrd, V> {
    keys: Vec<T>,
    values: Vec<V>,
    children: Vec<Rc<RefCell<Node<T, V>>>>,
    parent: Option<Rc<RefCell<Node<T, V>>>>,
    // max_keys: i32,
}
/// BTree
///
pub struct BTree<T: PartialOrd, V> {
    max_keys_per_node: usize,
    root: Rc<RefCell<Node<T, V>>>,
}

impl<T: PartialOrd, V> BTree<T, V> {
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
        if self.root.borrow().keys.len() <= self.max_keys_per_node / 2 - 1 {
            let _ = BTree::insert_key_in_node(&mut self.root, key, value, self.max_keys_per_node);
            return;
        }
        let _ = BTree::traverse_insert(&mut self.root, key, value, self.max_keys_per_node);
    }

    // Returns an Err when the key does not exist
    pub fn remove(&mut self, _key: T) -> Result<V, ()> {
        Err(())
    }

    pub fn exists(&self, key: T) -> bool {
        // TODO: traverse_search can return an Err
        let does_exist = self.traverse_search(&self.root, key).unwrap();
        match does_exist {
            Some(_) => return true,
            None => return false,
        };
    }

    pub fn get(&self, key: T) -> Option<&V> {
        let does_exist = self.traverse_search(&self.root, key).unwrap();
        match does_exist {
            Some((_, value)) => return Some(value),
            None => return None,
        };
    }

    fn traverse_insert(
        current_node: &mut Rc<RefCell<Node<T, V>>>,
        key: T,
        value: V,
        max_keys_per_node: usize,
    ) -> Result<(), &'static str> {
        if current_node.borrow().children.len() == 0 {
            // insert key in current node
            let result =
                BTree::insert_key_in_node(&mut current_node, key, value, max_keys_per_node);
            match result {
                Ok(_) => return Ok(()),
                Err(e) => return Err(e),
            }
        }
        for (i, current_key) in current_node.borrow().keys.iter().enumerate() {
            if key == *current_key {
                // TODO: define key found error
                return Err("Key already exists");
            }
            if key < *current_key {
                let mut child_to_traverse = Rc::clone(&mut current_node.children[i]);
                let result =
                    BTree::traverse_insert(&mut child_to_traverse, key, value, max_keys_per_node);
                match result {
                    Ok(_) => return Ok(()),
                    Err(e) if e == "Node is full" => BTree::split_node(
                        Rc::clone(current_node),
                        &mut child_to_traverse,
                        key,
                        value,
                        max_keys_per_node,
                    ),
                    Err(e) => return Err(e),
                };
            }
        }
        let mut child_to_traverse = Rc::clone(&mut current_node.children[current_node.keys.len()]);
        BTree::traverse_insert(child_to_traverse, key, value, max_keys_per_node)
    }

    fn split_node(
        current_node: &mut Rc<RefCell<Node<T, V>>>,
        child_to_split: &mut Rc<Node<T, V>>,
        key: T,
        value: V,
        max_keys_per_node: usize,
    ) -> Result<(), &'static str> {
        let mut new_right_node = Node {
            keys: Vec::new(),
            values: Vec::new(),
            children: Vec::new(),
            parent: Some(Rc::clone(current_node)),
        };
        for i in max_keys_per_node / 2..max_keys_per_node {
            new_right_node.keys.push(child_to_split.keys[i]);
            new_right_node.values.push(child_to_split.values[i]);
            new_right_node
                .children
                .push(Rc::clone(&child_to_split.children[i]));
        }
        child_to_split.keys.truncate(max_keys_per_node / 2);
        child_to_split.values.truncate(max_keys_per_node / 2);
        child_to_split.children.truncate(max_keys_per_node / 2);

        Ok(())
    }

    fn insert_key_in_node(
        current_node: &mut Rc<RefCell<Node<T, V>>>,
        key: T,
        value: V,
        max_keys_per_node: usize,
    ) -> Result<&mut Rc<RefCell<Node<T, V>>>, &'static str> {
        let mut current_node = current_node.borrow_mut();
        if current_node.keys.len() >= max_keys_per_node {
            // Node is full, need to split
            // TODO: define node full error
            Err("Node is full")
        } else {
            for i in 0..current_node.keys.len() {
                if key < current_node.keys[i] {
                    current_node.keys.insert(i, key);
                    current_node.values.insert(i, value);
                    return Ok(current_node);
                }
            }
            current_node.keys.push(key);
            current_node.values.push(value);
            return Ok(current_node);
        }
    }

    fn traverse_search<'a>(
        &'a self,
        current_node: &'a Node<T, V>,
        key: T,
    ) -> Result<Option<(&'a Node<T, V>, &'a V)>, ()> {
        if current_node.children.len() > 0 {
            self.iterate_over_node_with_children(current_node, key)
        } else {
            self.iterate_over_node_without_children(current_node, key)
        }
    }

    fn iterate_over_node_with_children<'a>(
        &'a self,
        current_node: &'a Node<T, V>,
        key: T,
    ) -> Result<Option<(&'a Node<T, V>, &'a V)>, ()> {
        for (i, current_key) in current_node.keys.iter().enumerate() {
            if key == *current_key {
                return Ok(Some((current_node, &current_node.values[i])));
            }
            if key < *current_key {
                return self.traverse_search(&current_node.children[i], key);
            }
        }
        Err(())
    }

    fn iterate_over_node_without_children<'a>(
        &'a self,
        current_node: &'a Node<T, V>,
        key: T,
    ) -> Result<Option<(&'a Node<T, V>, &'a V)>, ()> {
        for (i, current_key) in current_node.keys.iter().enumerate() {
            if key == *current_key {
                return Ok(Some((current_node, &current_node.values[i])));
            }
        }
        return Ok(None);
    }
}
