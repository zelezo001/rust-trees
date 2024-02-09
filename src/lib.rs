use std::cmp::Ordering;

pub mod rb;
pub mod avl;


type Child<K, V, I> = Option<Box<Node<K, V, I>>>;

struct Node<K: Ord, V, M> {
    key: K,
    value: V,
    left_child: Child<K, V, M>,
    right_child: Child<K, V, M>,
    metadata: M, // for data used in balancing algorithm
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Side {
    Left,
    Right,
}

impl Side {
    fn other(&self) -> Self {
        match self {
            Side::Left => Side::Right,
            Side::Right => Side::Left
        }
    }
}

// BST operations that does not change tree
impl<K: Ord, V, M> Node<K, V, M> {
    fn find(&self, key: &K) -> Option<&V> {
        let mut root = self;
        loop {
            match key.cmp(&root.key) {
                Ordering::Less => match &root.left_child {
                    None => {
                        return None;
                    }
                    Some(child) => {
                        root = child;
                    }
                },
                Ordering::Greater => match &root.right_child {
                    None => {
                        return None;
                    }
                    Some(child) => {
                        root = child;
                    }
                },
                Ordering::Equal => {
                    return Some(&root.value);
                }
            }
        }
    }

    fn min(&self) -> (&K, &V) {
        let mut min = self;
        while let Some(right_child) = &min.right_child {
            min = right_child;
        }

        return (&min.key, &min.value);
    }

    fn max(&self) -> (&K, &V) {
        let mut max = self;
        while let Some(right_child) = &max.right_child {
            max = right_child;
        }

        return (&max.key, &max.value);
    }

    // finds smallest node with key larger than given key
    fn next(&self, key: &K) -> Option<(&K, &V)> {
        let mut root = self;
        let mut last_greater = None;
        loop {
            match key.cmp(&root.key) {
                Ordering::Less => {
                    last_greater = Some(root);
                    match &root.left_child {
                        None => {
                            // current node is leaf, next node is current one
                            return last_greater.and_then(|node| { Some((&node.key, &node.value)) });
                        }
                        Some(child) => {
                            // we take step left in the tree, if next node is given one without right child this is next larger
                            root = child;
                        }
                    };
                }
                // searched node is larger so we must look in right subtree
                Ordering::Greater => match &root.right_child {
                    None => {
                        return None;
                    }
                    Some(child) => {
                        root = child;
                    }
                },
                // we found node with given key
                Ordering::Equal => {
                    return match &root.right_child {
                        None => {
                            // node has no children with larger nodes, smallest node is last larger one
                            // if last_greater is none, given is largest in the whole tree
                            last_greater.and_then(|node| { Some((&node.key, &node.value)) })
                        }
                        Some(right_child) => {
                            // node has children with larger nodes, smallest of them is next node
                            Some(right_child.min())
                        }
                    };
                }
            };
        };
    }
}

pub struct Tree<K: Ord, V, I> {
    root: Child<K, V, I>,
}

impl<K: Ord, V, I> Tree<K, V, I> {
    pub fn find(&self, value: &K) -> Option<&V> {
        match &self.root {
            None => {
                None
            }
            Some(node) => {
                node.find(value)
            }
        }
    }

    pub fn min(&self) -> Option<(&K, &V)> {
        match &self.root {
            None => {
                None
            }
            Some(node) => {
                Some(node.min())
            }
        }
    }

    pub fn max(&self) -> Option<(&K, &V)> {
        match &self.root {
            None => {
                None
            }
            Some(node) => {
                Some(node.max())
            }
        }
    }

    pub fn next(&self, value: &K) -> Option<(&K, &V)> {
        match &self.root {
            None => {
                None
            }
            Some(node) => {
                node.next(value)
            }
        }
    }
}