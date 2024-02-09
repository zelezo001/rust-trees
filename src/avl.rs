use std::cmp::{Ordering};
use std::fmt::Debug;
use std::mem;
use std::ops::{Neg};
use super::Side;

#[derive(Debug, Clone, Copy, PartialEq)]
enum HeightChange {
    Increased,
    Decreased,
    Unchanged,
}

// metadata < 0 left child is higher
// metadata > 0 right child is higher
type Node<K, V> = super::Node<K, V, i8>;
type BoxedNode<K, V> = Box<Node<K, V>>;
type Child<K, V> = super::Child<K, V, i8>;

fn new_node<K: Ord, V>(key: K, value: V) -> Child<K, V> {
    Some(Box::new(Node {
        key,
        value,
        left_child: None,
        right_child: None,
        metadata: 0,
    }))
}

fn abs<V: Neg<Output=V> + PartialOrd<V> + Copy>(value: V) -> V {
    if value < -value {
        return -value;
    }
    value
}

impl<K: Ord, V> Node<K, V> {
    fn pop_smallest_node(mut node: BoxedNode<K, V>) -> (Child<K, V>, BoxedNode<K, V>, HeightChange) {
        match node.left_child.take() {
            // cannot continue, return current node
            None => {
                // there can be some nodes in right subtree, so we must return them
                let right_child = node.right_child.take();
                (right_child, node, HeightChange::Decreased)
            }
            // node has left child and therefore is not smallest
            Some(child) => {
                let (left, popped, mut height_change) = Self::pop_smallest_node(child);
                node.left_child = left;
                // removing child in subtree could have affected height, we must check AVL rules
                height_change = node.handle_child_change(height_change, Side::Left);
                (Some(node), popped, height_change)
            }
        }
    }

    fn remove(mut self, key: &K) -> (Child<K, V>, HeightChange, Option<(K, V)>) {
        match key.cmp(&self.key) {
            Ordering::Equal => {
                let (has_left_child, has_right_child) = (self.left_child.is_some(), self.right_child.is_some());
                // node has 2 children, we can replace current node with next node in inorder succession
                if has_right_child && has_left_child {
                    let (right_child, mut replacement, mut change) = Self::pop_smallest_node(self.right_child.unwrap());
                    self.right_child = right_child;

                    mem::swap(&mut replacement.key, &mut self.key);
                    mem::swap(&mut replacement.value, &mut self.value);

                    change = self.handle_child_change(change, Side::Right);
                    (Some(Box::new(self)), change, Some((replacement.key, replacement.value)))
                }
                // node has one child, we can replace current node with it
                else if has_right_child {
                    return (Some(self.right_child.take().unwrap()), HeightChange::Decreased, Some((self.key, self.value)));
                } else if has_left_child {
                    return (Some(self.left_child.take().unwrap()), HeightChange::Decreased, Some((self.key, self.value)));
                }
                // node has no children,
                else {
                    return (None, HeightChange::Decreased, Some((self.key, self.value)));
                }
            }
            // value is not in current node, we will search it in corresponding child if it exists
            Ordering::Greater => {
                match self.right_child.take() {
                    Some(child) => {
                        let (child, change, value) = child.remove(key);
                        self.right_child = child;
                        let change = self.handle_child_change(change, Side::Right);
                        (Some(Box::new(self)), change, value)
                    }
                    None => {
                        (Some(Box::new(self)), HeightChange::Unchanged, None)
                    }
                }
            }
            Ordering::Less => {
                match self.left_child.take() {
                    Some(child) => {
                        let (child, change, value) = child.remove(key);
                        self.left_child = child;
                        let change = self.handle_child_change(change, Side::Left);
                        (Some(Box::new(self)), change, value)
                    }
                    None => {
                        (Some(Box::new(self)), HeightChange::Unchanged, None)
                    }
                }
            }
        }
    }

    fn insert(&mut self, key: K, value: V) -> HeightChange {
        let affected_child_side: Side;
        let mut affected_child_change = HeightChange::Increased;
        match key.cmp(&self.key) {
            Ordering::Equal => {
                self.key = key;
                self.value = value;
                return HeightChange::Unchanged;
            }
            Ordering::Less => {
                affected_child_side = Side::Left;
                match &mut self.left_child {
                    None => {
                        self.left_child = new_node(key, value);
                    }
                    Some(child) => {
                        affected_child_change = child.insert(key, value);
                    }
                };
            }
            Ordering::Greater => {
                affected_child_side = Side::Right;
                match &mut self.right_child {
                    None => {
                        self.right_child = new_node(key, value);
                    }
                    Some(child) => {
                        affected_child_change = child.insert(key, value);
                    }
                }
            }
        }

        self.handle_child_change(affected_child_change, affected_child_side)
    }

    fn handle_child_change(
        &mut self,
        affected_child_change: HeightChange,
        affected_child_side: Side,
    ) -> HeightChange {
        match affected_child_change {
            HeightChange::Unchanged => {
                return HeightChange::Unchanged;
            }
            HeightChange::Increased => {
                self.metadata += match affected_child_side {
                    Side::Left => -1,
                    Side::Right => 1,
                };
                // other subtree was higher than affected one, height change made their height same
                if self.metadata == 0 {
                    return HeightChange::Unchanged;
                }
                // subtrees had same size before change so now
                if abs(self.metadata) == 1 {
                    return HeightChange::Increased;
                }
            }
            HeightChange::Decreased => {
                self.metadata += match affected_child_side {
                    Side::Left => 1,
                    Side::Right => -1,
                };
                // subtree was higher than affected one, height change made their height same but also decreased height of tree
                if self.metadata == 0 {
                    return HeightChange::Decreased;
                }
                // subtrees had same height, as height of tree is equal to height of higher subtree, height did not change
                if abs(self.metadata) == 1 {
                    return HeightChange::Unchanged;
                }
            }
        }
        // balance factor is |2|, tree must be rebalanced
        self.balance();
        if affected_child_change == HeightChange::Decreased && self.metadata == 0 {
            HeightChange::Decreased
        } else {
            // rotations absorbed height change
            HeightChange::Unchanged
        }
    }

    fn balance(&mut self) {
        // tree is left leaning
        if self.metadata == -2 {
            if self.left_child.as_ref().unwrap().metadata <= 0 {
                // simple rotation to the right is enough
                self.rotate_right();
            } else {
                // we don't know if LL child exists, more complex rotation is needed
                self.rotate_left_right();
            }
        }
        // tree is right leaning
        else if self.metadata == 2 {
            if self.right_child.as_ref().unwrap().metadata >= 0 {
                // simple rotation to the left is enough
                self.rotate_left();
            } else {
                // we don't know if RR child exists, more complex rotation is needed
                self.rotate_right_left();
            }
        } else {
            unreachable!();
        }
    }

    // Rotates right-heavy tree with left-leaning right child
    //     a               c
    //    / \             / \
    //   W   b           a   b
    //      / \    =>   / \ / \
    //     c   X       X  Y Z  W
    //    / \
    //   Y   Z
    fn rotate_right_left(&mut self) {
        let mut right = self.right_child.take().unwrap(); // b
        let mut new_root = right.left_child.take().unwrap(); // c
        right.left_child = new_root.right_child.take(); // reassign Z
        self.right_child = new_root.left_child.take(); // reassign Y

        mem::swap(self, &mut new_root); // c is now root and a new_root
        self.left_child = Some(new_root); // reassign a to c
        self.right_child = Some(right); // reassign b to c

        // if c was not balanced we must reflect it new parents of Y, Z
        // from properties of AVL tree we know that height of W, X and Y XOR Z are same
        let (mut left_child_balance_factor, mut right_child_balance_factor) = (0, 0);
        if self.metadata == 1 {
            // Z > Y => Z > W => balance of a is -1
            left_child_balance_factor = -1;
        } else if self.metadata == -1 {
            // Y > Z => Y > X => balance of b is 1
            right_child_balance_factor = 1;
        }
        self.right_child.as_mut().unwrap().metadata = right_child_balance_factor;
        self.left_child.as_mut().unwrap().metadata = left_child_balance_factor;
        self.metadata = 0;
    }

    // Rotates left-heavy tree with right-leaning left child
    //        a              c
    //       / \            / \
    //      b   W          b   a
    //     / \      =>    / \ / \
    //    X   c          X  Y Z  W
    //       / \
    //      Y   Z
    fn rotate_left_right(&mut self) {
        let mut left = self.left_child.take().unwrap(); // b
        let mut new_root = left.right_child.take().unwrap(); // c
        self.left_child = new_root.right_child.take(); // reassign Z
        left.right_child = new_root.left_child.take(); // reassign Y

        mem::swap(self, &mut new_root); // c is now root and a new_root
        self.right_child = Some(new_root); // reassign a to c
        self.left_child = Some(left); // reassign b to c

        // if c was not balanced we must reflect it new parents of Y, Z
        // from properties of AVL tree we know that height of W, X and Y XOR Z are same
        let (mut left_child_balance_factor, mut right_child_balance_factor) = (0, 0);

        if self.metadata == 1 {
            left_child_balance_factor = -1;
        } else if self.metadata == -1 {
            right_child_balance_factor = 1;
        }

        self.right_child.as_mut().unwrap().metadata = right_child_balance_factor;
        self.left_child.as_mut().unwrap().metadata = left_child_balance_factor;
        self.metadata = 0;
    }

    // Rotates right-heavy tree with balanced or right-leaning right child
    //        a                b
    //       / \              / \
    //      X   b            a   Y
    //         / \    =>    / \
    //        Z   Y        X  Z
    fn rotate_left(&mut self) {
        let mut new_root = self.right_child.take().unwrap();
        self.right_child = new_root.left_child.take();
        if new_root.metadata == 0 {
            // height of Z and Y is same => a will be right-leaning and b left-leaning
            new_root.metadata = -1;
            self.metadata = 1;
        } else {
            // height of Y = 1 + height of Z (guaranteed by AVL tree properties and check before calling rotate_right)
            // => W and Y have same height => b and a are balanced
            new_root.metadata = 0;
            self.metadata = 0;
        }
        mem::swap(self, &mut new_root);
        self.left_child = Some(new_root);
    }

    // Rotates left-heavy tree with balanced or left-leaning left child
    //        a            b
    //       / \          / \
    //      b   X        Z   a
    //     / \      =>      / \
    //    Z   Y            Y   X
    fn rotate_right(&mut self) {
        let mut new_root = self.left_child.take().unwrap(); // b
        self.left_child = new_root.right_child.take(); // assign Y to a
        if new_root.metadata == 0 {
            // height of Z and Y is same => a will be left-leaning and b right-leaning
            new_root.metadata = 1;
            self.metadata = -1;
        } else {
            // height of Z = 1 + height of Y (guaranteed by AVL tree properties and check before calling rotate_right)
            // => W and Y have same height => b and a are balanced
            new_root.metadata = 0;
            self.metadata = 0;
        }
        mem::swap(self, &mut new_root); // switch a and b
        self.right_child = Some(new_root); // assign a to b
    }
}

pub type AVL<K, V> = super::Tree<K, V, i8>;

impl<K: Ord, V> AVL<K, V> {
    pub fn new() -> Self {
        return AVL { root: None };
    }

    pub fn remove(&mut self, key: &K) -> Option<(K, V)> {
        match self.root.take() {
            None => { None }
            Some(node) => {
                let returned_value;
                (self.root, _, returned_value) = node.remove(key);
                returned_value
            }
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        match &mut self.root {
            None => {
                self.root = new_node(key, value);
            }
            Some(node) => {
                node.insert(key, value);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::max;
    use super::*;

    #[test]
    fn test_inserting_and_deleting_keeps_tree_balanced() {
        let vec: Vec<u32> = (0..1000).collect();
        let mut tree = AVL::new();
        for (i, j) in vec.iter().enumerate() {
            tree.insert(j.clone(), j.clone());
            check_tree(tree.root.as_ref().unwrap(), (i + 1) as u32);
        }
        for j in vec.iter() {
            assert_eq!(Some(j), tree.find(j));
        }
        assert!(tree.root.is_some());
        let mut size = vec.len();
        for j in vec.iter() {
            assert_eq!(Some((j.clone(), j.clone())), tree.remove(j));
            size -= 1;
            if size > 0 {
                check_tree(tree.root.as_ref().unwrap(), size as u32);
            } else {
                assert!(tree.root.is_none());
            }
        }
    }

    fn check_tree<K: Ord, V>(tree: &Box<Node<K, V>>, expected_size: u32) {
        let (_, size) = check_balance_factors(tree);
        assert_eq!(expected_size, size);
    }

    fn check_balance_factors<K: Ord, V>(tree: &Box<Node<K, V>>) -> (u32, u32) {
        let (left, left_tree_size) = match &tree.left_child {
            None => (0, 0),
            Some(child) => check_balance_factors(child),
        };

        let (right, right_tree_size) = match &tree.right_child {
            None => (0, 0),
            Some(child) => check_balance_factors(child),
        };

        assert_eq!(tree.metadata as i64, right as i64 - left as i64);

        (max(left, right) + 1, 1 + right_tree_size + left_tree_size)
    }
}
