use std::rc::{Rc, Weak};
// Perhaps T and V need some traits?
struct Node<T: PartialOrd, V> {
    keys: Vec<T>,
    values: Vec<V>,
    children: Option<Vec<Rc<Node<T, V>>>>,
    parent: Option<Weak<Node<T, V>>>,
    // max_keys: i32,
}
/// BTree
///
pub struct BTree<T: PartialOrd, V> {
    max_keys_per_node: usize,
    root: Rc<Node<T, V>>,
}

impl<T: PartialOrd, V> BTree<T, V> {
    pub fn new(first_key: T, first_value: V, max_keys_per_node: usize) -> Self {
        let root = Rc::new(Node {
            keys: vec![first_key],
            values: vec![first_value],
            children: Some(Vec::new()),
            parent: None,
            // max_keys: max_keys_per_node,
        });
        let tree = BTree {
            max_keys_per_node,
            root,
        };
        tree
    }

    // Returns an Err when the key already exists
    pub fn insert(&mut self, key: T, value: V) {
        self.insert_internal(key, value);
    }
    fn insert_internal(&mut self, key: T, value: V) {
        let _ = BTree::traverse_insert(
            &mut Rc::get_mut(&mut self.root).unwrap(),
            key,
            value,
            self.max_keys_per_node,
        );
    }

    // Returns an Err when the key does not exist
    pub fn remove(&mut self, _key: T) -> Result<V, ()> {
        Err(())
    }

    pub fn exists(&self, key: T) -> bool {
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

    fn traverse_insert(current_node: &mut Node<T, V>, key: T, value: V, max_keys_per_node: usize) {
        match &mut current_node.children {
            // traverse further?
            Some(children) => {
                let new_current_node = Rc::get_mut(&mut children[0]).unwrap();
                Self::traverse_insert(new_current_node, key, value, max_keys_per_node);
            }
            // Insert here
            None => {
                BTree::insert_key_in_node(current_node, key, value, max_keys_per_node);
            }
        };
    }
    fn insert_key_in_node(
        current_node: &mut Node<T, V>,
        key: T,
        value: V,
        max_keys_per_node: usize,
    ) -> Result<&mut Node<T, V>, ()> {
        if current_node.keys.len() >= max_keys_per_node {
            // Node is full, need to split
            Err(())
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
        match current_node.children {
            Some(_) => return self.iterate_over_node_with_children(current_node, key),
            None => return self.iterate_over_node_without_children(current_node, key),
        };
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
                return self.traverse_search(&current_node.children.as_ref().unwrap()[i], key);
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
