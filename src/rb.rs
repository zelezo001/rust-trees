use std::cmp::Ordering;
use std::fmt::Debug;
use std::mem;
use super::Side;


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
    Black,
    Red,
}

type InsertRotation = Option<Side>;
type Node<K, V> = super::Node<K, V, Color>;
type BoxedNode<K, V> = Box<Node<K, V>>;
type Child<K, V> = super::Child<K, V, Color>;
pub type RedBlack<K, V> = super::Tree<K, V, Color>;

impl<K: Ord, V> Node<K, V> {
    fn insert(&mut self, key: K, value: V) {
        self.insert_recursively(key, value);
        // after recursive insertion we can get red root and red children, we can fix this with painting root black
        self.metadata = Color::Black;
    }

    fn insert_recursively(&mut self, key: K, value: V) -> InsertRotation {
        let rotation: InsertRotation;
        match key.cmp(&self.key) {
            Ordering::Equal => {
                self.key = key;
                self.value = value;
                None
            }
            Ordering::Less => {
                match &mut self.left_child {
                    None => {
                        self.left_child = new_node(key, value, Color::Red);
                        return self.resolve_rotation(Color::Red, Side::Left);
                    }
                    Some(child) => {
                        rotation = child.insert_recursively(key, value);
                    }
                };
                return self.handle_insert_rotation(rotation, Side::Left);
            }
            Ordering::Greater => {
                match &mut self.right_child {
                    None => {
                        self.right_child = new_node(key, value, Color::Red);
                        return self.resolve_rotation(Color::Red, Side::Right);
                    }
                    Some(child) => {
                        rotation = child.insert_recursively(key, value);
                    }
                }
                return self.handle_insert_rotation(rotation, Side::Right);
            }
        }
    }

    fn handle_insert_rotation(&mut self, rotation: InsertRotation, child_side: Side) -> InsertRotation {
        if let Some(grand_child_side) = rotation {
            // if sibling is red, we can paint him black and self red, which restores balance in number of black nodes
            if let Some(sibling) = self.another_child(child_side) {
                if sibling.metadata == Color::Red {
                    sibling.metadata = Color::Black;
                    if let Some(child) = self.child(child_side) {
                        child.metadata = Color::Black;
                    }
                    self.metadata = Color::Red;
                    self.resolve_rotation(Color::Red, child_side)
                } else {
                    self.rotate(child_side, grand_child_side);
                    None
                }
            } else {
                self.rotate(child_side, grand_child_side);
                None
            }
        } else {
            None
        }
    }

    fn resolve_rotation(&self, child_color: Color, child_side: Side) -> InsertRotation {
        // red node has red child, which violates tree rules, rotation is needed
        if child_color == Color::Red && self.metadata == Color::Red {
            Some(child_side)
        } else {
            None
        }
    }

    fn rotate(&mut self, child_side: Side, grandchild_side: Side) {
        if grandchild_side != child_side {
            self.child(child_side).as_mut().unwrap().rotate_from(grandchild_side);
        }

        self.child(child_side).as_mut().unwrap().metadata = Color::Black;
        self.metadata = Color::Red;
        self.rotate_from(child_side);
    }

    fn another_child(&mut self, child_side: Side) -> &mut Child<K, V> {
        self.child(child_side.other())
    }

    fn child(&mut self, child_side: Side) -> &mut Child<K, V> {
        match child_side {
            Side::Left => &mut self.left_child,
            Side::Right => &mut self.right_child
        }
    }

    fn rotate_from(&mut self, side: Side) {
        match side {
            Side::Left => self.rotate_right(),
            Side::Right => self.rotate_left()
        }
    }

    fn rotate_to(&mut self, side: Side) {
        match side {
            Side::Left => self.rotate_left(),
            Side::Right => self.rotate_right()
        }
    }


    // Rotates tree to the left
    //        a                b
    //       / \              / \
    //      W   b            a   Y
    //         / \    =>    / \
    //        Z   Y        W   Z
    fn rotate_left(&mut self) {
        let mut new_self = self.right_child.take().unwrap(); // takes b
        self.right_child = new_self.left_child.take(); // reassign Z
        mem::swap(self, &mut new_self);
        self.left_child = Some(new_self); // takes a to b
    }

    // Rotates tree to the right
    //        a            b
    //       / \          / \
    //      b   X        Z   a
    //     / \      =>      / \
    //    Z   Y            Y   X
    fn rotate_right(&mut self) {
        let mut new_self = self.left_child.take().unwrap(); // takes b
        self.left_child = new_self.right_child.take(); // reassign Y
        mem::swap(self, &mut new_self);
        self.right_child = Some(new_self); // takes a to b
    }

    fn pop_smallest_node(mut node: BoxedNode<K, V>) -> (Child<K, V>, BoxedNode<K, V>, bool) {
        match node.left_child.take() {
            None => {
                match node.right_child.take() {
                    None => {
                        match node.metadata {
                            Color::Red => {
                                (None, node, false)
                            }
                            Color::Black => {
                                (None, node, true)
                            }
                        }
                    }
                    Some(mut right_child) => {
                        right_child.metadata = Color::Black;
                        (Some(right_child), node, false)
                    }
                }
            }
            Some(child) => {
                let (left, popped, mut check_needed) = Self::pop_smallest_node(child);
                node.left_child = left;
                if check_needed {
                    check_needed = node.check_imbalance_after_delete(Side::Left);
                }
                (Some(node), popped, check_needed)
            }
        }
    }

    fn remove(self, value: &K) -> (Child<K, V>, Option<(K, V)>) {
        let (mut node, removed, _) = self.remove_recursively(value);
        if let Some(node) = node.as_mut() {
            // after recursive insertion we can get red root and red children, we can fix this with painting root black
            node.metadata = Color::Black;
        }

        return (node, removed);
    }

    fn remove_recursively(mut self, key: &K) -> (Child<K, V>, Option<(K, V)>, bool) {
        match key.cmp(&self.key) {
            Ordering::Equal => {
                let (has_left_child, has_right_child) = (self.left_child.is_some(), self.right_child.is_some());
                if has_right_child && has_left_child {
                    let (right, mut replacement, mut check_needed) = Self::pop_smallest_node(self.right_child.take().unwrap());

                    // replace self with next node in inorder succession
                    mem::swap(&mut replacement.key, &mut self.key);
                    mem::swap(&mut replacement.value, &mut self.value);

                    self.right_child = right;
                    if check_needed {
                        check_needed = self.check_imbalance_after_delete(Side::Right);
                    }
                    (Some(Box::new(self)), Some((replacement.key, replacement.value)), check_needed)
                }
                // node has one child, we can replace current node with it
                // painting child black ensures red node does not have red child
                // number of black nodes in paths to leafs is not violated as only child cannot be black
                else if has_right_child {
                    let mut child = self.right_child.take();
                    if let Some(child) = &mut child {
                        child.metadata = Color::Black;
                    }
                    (child, Some((self.key, self.value)), false)
                } else if has_left_child {
                    let mut child = self.left_child.take();
                    if let Some(child) = &mut child {
                        child.metadata = Color::Black;
                    }
                    (child, Some((self.key, self.value)), false)
                } else {
                    match self.metadata {
                        Color::Red => {
                            (None, Some((self.key, self.value)), false)
                        }
                        Color::Black => {
                            (None, Some((self.key, self.value)), true)
                        }
                    }
                }
            }
            // value is not in current node, we will search it in corresponding child if it exists
            Ordering::Greater => {
                match self.right_child.take() {
                    Some(child) => {
                        let (child, value, mut check_needed) = child.remove_recursively(key);
                        self.right_child = child;
                        if check_needed {
                            check_needed = self.check_imbalance_after_delete(Side::Right);
                        }
                        (Some(Box::new(self)), value, check_needed)
                    }
                    None => {
                        (Some(Box::new(self)), None, false)
                    }
                }
            }
            Ordering::Less => {
                match self.left_child.take() {
                    Some(child) => {
                        let (child, value, mut check_needed) = child.remove_recursively(key);
                        self.left_child = child;
                        if check_needed {
                            check_needed = self.check_imbalance_after_delete(Side::Left);
                        }
                        (Some(Box::new(self)), value, check_needed)
                    }
                    None => {
                        (Some(Box::new(self)), None, false)
                    }
                }
            }
        }
    }

    // path from root to leafs on changed_child_side has one less black nodes than path to other leafs
    // we must apply appropriate repainting to restore balance
    fn check_imbalance_after_delete(&mut self, changed_child_side: Side) -> bool {
        let is_red = self.metadata == Color::Red;
        let sibling = self.another_child(changed_child_side).as_mut().unwrap();
        if sibling.metadata == Color::Red { // sibling is red, his children and self must be black
            self.balance_red_sibling(changed_child_side);
            false
        } else {
            let same_side_nephew_is_red = !Self::is_black(sibling.child(changed_child_side));
            let other_side_nephew_is_red = !Self::is_black(sibling.another_child(changed_child_side));
            if other_side_nephew_is_red {
                self.balance_other_side_nephew_is_red(changed_child_side);
                false
                // other side nephew is black, we need to do rotations on sibling first to be able to balance tree
            } else if same_side_nephew_is_red {
                self.balance_same_side_nephew_is_red(changed_child_side);
                false
            } else if is_red {
                self.balance_red_node(changed_child_side);
                false
            } else {
                // self, both childs and children of sibling are black
                // repainting sibling restores number of black nodes in subtrees, additional checks are needed in upper layers of tree
                sibling.metadata = Color::Red;
                true
            }
        }
    }

    fn balance_red_sibling(&mut self, changed_child_side: Side) {
        // rotating to changed side and paining new root black (previous sibling) and old one red
        // does not change number of black nodes in path to leafs in other side subtree
        self.rotate_to(changed_child_side);
        self.metadata = Color::Black;
        let previous_self = self.child(changed_child_side).as_mut().unwrap();
        previous_self.metadata = Color::Red;
        // but previous_self changed side child is still missing one black node
        // we can balance previous_self so changed side child does comply with required number of black nodes
        let sibling = previous_self.child(changed_child_side.other()).as_mut().unwrap();
        if Self::is_red(sibling.child(changed_child_side.other())) {
            previous_self.balance_other_side_nephew_is_red(changed_child_side);
        } else if Self::is_red(sibling.child(changed_child_side)) {
            previous_self.balance_same_side_nephew_is_red(changed_child_side);
        } else {
            previous_self.balance_red_node(changed_child_side);
        }
    }

    // self looks like graph below where big B are subtrees with same number of black nodes
    // repainting color of root and sibling b of changed child will balance subtree
    //        R                B
    //       / \              / \
    //      B   b            B   r
    //         / \    =>        / \
    //        B   B            B   B
    fn balance_red_node(&mut self, changed_child_side: Side) {
        self.metadata = Color::Black;
        self.child(changed_child_side.other()).as_mut().unwrap().metadata = Color::Red;
    }

    // self looks like graph below where changed child and R are subtrees with same number of black nodes
    // C/c indicates, that color can be both red or black, b is sibling of changed child
    // rotation and repainting should restore balance between sides
    //        C                c
    //       / \              / \
    //      B   b            B   B
    //         / \    =>    / \
    //        R   B        B   B
    fn balance_other_side_nephew_is_red(&mut self, side: Side) {
        let color = self.metadata;
        self.metadata = Color::Black;
        self.rotate_to(side);
        self.metadata = color;
        self.child(side.other()).as_mut().unwrap().metadata = Color::Black;
    }

    fn balance_same_side_nephew_is_red(&mut self, side: Side) {
        // this rotation and repainting won't balance tree it will puts tree in state that it can be balanced with balance_other_side_nephew_is_red
        let sibling = self.child(side.other()).as_mut().unwrap();
        sibling.metadata = Color::Red;
        sibling.rotate_to(side.other());
        sibling.metadata = Color::Black;
        self.balance_other_side_nephew_is_red(side);
    }

    fn is_black(node: &Child<K, V>) -> bool {
        !Self::is_red(node)
    }
    fn is_red(node: &Child<K, V>) -> bool {
        node.as_ref().is_some_and(|x| { x.metadata == Color::Red })
    }
}


fn new_node<K: Ord, V>(key: K, value: V, color: Color) -> Child<K, V> {
    Some(Box::new(Node {
        key,
        value,
        left_child: None,
        right_child: None,
        metadata: color,
    }))
}

impl<K: Ord, V> RedBlack<K, V> {
    pub fn new() -> Self {
        return RedBlack { root: None };
    }

    pub fn remove(&mut self, value: &K) -> Option<(K, V)> {
        match self.root.take() {
            None => { None }
            Some(node) => {
                let returned_value;
                (self.root, returned_value) = node.remove(value);
                returned_value
            }
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        match &mut self.root {
            None => {
                self.root = new_node(key, value, Color::Black);
            }
            Some(node) => {
                node.insert(key, value);
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inserting_and_deleting_keeps_tree_balanced() {
        let vec: Vec<u32> = (0..1000).collect();
        let mut tree = RedBlack::new();
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
        assert_eq!(tree.metadata, Color::Black);
        let (_, size) = check_tree_recursively(tree);
        assert_eq!(size, expected_size);
    }

    fn check_tree_recursively<K: Ord, V>(tree: &Box<Node<K, V>>) -> (u32, u32) {
        let (left, left_children) = match &tree.left_child {
            None => (1, 0),
            Some(child) => {
                check_tree_recursively(child)
            }
        };

        let (right, right_children) = match &tree.right_child {
            None => (1, 0),
            Some(child) => {
                check_tree_recursively(child)
            }
        };

        assert_eq!(left, right);

        return if tree.metadata == Color::Black {
            (left + 1, right_children + left_children + 1)
        } else {
            assert_eq!(false, tree.left_child.as_ref().is_some_and(|x| { x.metadata == Color::Red }));
            assert_eq!(false, tree.right_child.as_ref().is_some_and(|x| { x.metadata == Color::Red }));
            (left, right_children + left_children + 1)
        };
    }
}