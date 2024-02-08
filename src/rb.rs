use std::cmp::Ordering;
use std::fmt::Debug;
use std::mem;

pub struct RedBlack<T: Ord> {
    root: Tree<T>,
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

#[derive(Debug, Clone, Copy, PartialEq)]
enum Color {
    Black,
    Red,
}

type InsertRotation = Option<Side>;
type BoxedNode<T> = Box<Node<T>>;
type Tree<T> = Option<BoxedNode<T>>;

#[derive(Debug)]
struct Node<T: Ord> {
    value: T,
    left_child: Tree<T>,
    right_child: Tree<T>,
    color: Color,
}

impl<T: Ord + Debug> Node<T> {
    fn find(&self, value: &T) -> Option<&T> {
        let mut root = self;
        loop {
            match value.cmp(&root.value) {
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

    fn min(&self) -> &T {
        let mut min = self;
        while let Some(right_child) = &min.right_child {
            min = right_child;
        }

        return &min.value;
    }

    fn max(&self) -> &T {
        let mut max = self;
        while let Some(right_child) = &max.right_child {
            max = right_child;
        }

        return &max.value;
    }

    fn next(&self, value: &T) -> Option<&T> {
        let mut root = self;
        let mut last_greater = None;
        loop {
            match value.cmp(&root.value) {
                Ordering::Less => {
                    last_greater = Some(root);
                    match &root.left_child {
                        None => {
                            return last_greater.and_then(|node| { Some(&node.value) });
                        }
                        Some(child) => {
                            root = child;
                        }
                    };
                }
                Ordering::Greater => match &root.right_child {
                    None => {
                        return None;
                    }
                    Some(child) => {
                        root = child;
                    }
                },
                Ordering::Equal => {
                    match &root.right_child {
                        None => {
                            return last_greater.and_then(|node| { Some(&node.value) });
                        }
                        Some(right_child) => {
                            return Some(right_child.max());
                        }
                    }
                }
            };
        };
    }

    fn insert(&mut self, value: T) {
        self.insert_recursively(value);
        self.color = Color::Black;
    }

    fn insert_recursively(&mut self, value: T) -> InsertRotation {
        let rotation: InsertRotation;
        match value.cmp(&self.value) {
            Ordering::Equal => {
                self.value = value;
                None
            }
            Ordering::Less => {
                match &mut self.left_child {
                    None => {
                        self.left_child = new_node(value, Color::Red);
                        return self.resolve_rotation(Color::Red, Side::Left);
                    }
                    Some(child) => {
                        rotation = child.insert_recursively(value);
                    }
                };
                return self.handle_insert_rotation(rotation, Side::Left);
            }
            Ordering::Greater => {
                match &mut self.right_child {
                    None => {
                        self.right_child = new_node(value, Color::Red);
                        return self.resolve_rotation(Color::Red, Side::Right);
                    }
                    Some(child) => {
                        rotation = child.insert_recursively(value);
                    }
                }
                return self.handle_insert_rotation(rotation, Side::Right);
            }
        }
    }

    fn handle_insert_rotation(&mut self, rotation: InsertRotation, child_side: Side) -> InsertRotation {
        if let Some(grand_child_side) = rotation {
            if let Some(uncle) = self.another_child(child_side) {
                if uncle.color == Color::Red {
                    uncle.color = Color::Black;
                    self.child(child_side).as_mut().unwrap().color = Color::Black;
                    self.color = Color::Red;
                } else {
                    self.rotate(child_side, grand_child_side);
                }
            } else {
                self.rotate(child_side, grand_child_side);
            }
        }

        let child_color = self.child(child_side).as_ref().unwrap().color;
        self.resolve_rotation(child_color, child_side)
    }

    fn resolve_rotation(&self, child_color: Color, child_side: Side) -> InsertRotation {
        if child_color == Color::Red && self.color == Color::Red {
            Some(child_side)
        } else {
            None
        }
    }

    fn rotate(&mut self, child_side: Side, grandchild_side: Side) {
        if grandchild_side != child_side {
            self.child(child_side).as_mut().unwrap().rotate_side(grandchild_side);
        }
        self.child(child_side).as_mut().unwrap().color = Color::Black;
        self.color = Color::Red;
        self.rotate_side(child_side);
    }

    fn another_child(&mut self, child_side: Side) -> &mut Tree<T> {
        self.child(child_side.other())
    }

    fn child(&mut self, child_side: Side) -> &mut Tree<T> {
        match child_side {
            Side::Left => &mut self.left_child,
            Side::Right => &mut self.right_child
        }
    }

    fn rotate_side(&mut self, side: Side) {
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

    fn rotate_left(&mut self) {
        let mut right = self.right_child.take().unwrap();
        self.right_child = right.left_child.take();
        mem::swap(self, &mut right);
        self.left_child = Some(right);
    }

    fn rotate_right(&mut self) {
        let mut left = self.left_child.take().unwrap();
        self.left_child = left.right_child.take();
        mem::swap(self, &mut left);
        self.right_child = Some(left);
    }

    fn pop_most_left(mut node: BoxedNode<T>) -> (Tree<T>, BoxedNode<T>, bool) {
        match node.left_child.take() {
            None => {
                match node.right_child.take() {
                    None => {
                        match node.color {
                            Color::Red => {
                                (None, node, false)
                            }
                            Color::Black => {
                                (None, node, true)
                            }
                        }
                    }
                    Some(mut right_child) => {
                        right_child.color = Color::Black;
                        (Some(right_child), node, false)
                    }
                }
            }
            Some(child) => {
                let (left, popped, mut check_needed) = Self::pop_most_left(child);
                node.left_child = left;
                if check_needed {
                    check_needed = node.check_imbalance_after_delete(Side::Left);
                }
                (Some(node), popped, check_needed)
            }
        }
    }

    fn remove(self, value: &T) -> (Tree<T>, Option<T>) {
        let (mut node, removed, _) = self.remove_recursively(value);
        if let Some(node) = node.as_mut() {
            node.color = Color::Black;
        }

        return (node, removed);
    }

    fn remove_recursively(mut self, value: &T) -> (Tree<T>, Option<T>, bool) {
        match value.cmp(&self.value) {
            Ordering::Equal => {
                let (has_left_child, has_right_child) = (self.left_child.is_some(), self.right_child.is_some());
                if has_right_child && has_left_child {
                    let (right, replacement, mut check_needed) = Self::pop_most_left(self.right_child.take().unwrap());
                    let value = mem::replace(&mut self.value, replacement.value);
                    self.right_child = right;
                    if check_needed {
                        check_needed = self.check_imbalance_after_delete(Side::Right);
                    }
                    (Some(Box::new(self)), Some(value), check_needed)
                } else if has_right_child {
                    let mut child = self.right_child.take();
                    if let Some(child) = &mut child {
                        child.color = Color::Black;
                    }
                    (child, Some(self.value), false)
                } else if has_left_child {
                    let mut child = self.left_child.take();
                    if let Some(child) = &mut child {
                        child.color = Color::Black;
                    }
                    (child, Some(self.value), false)
                } else {
                    match self.color {
                        Color::Red => {
                            (None, Some(self.value), false)
                        }
                        Color::Black => {
                            (None, Some(self.value), true)
                        }
                    }
                }
            }
            Ordering::Greater => {
                match self.right_child.take() {
                    Some(child) => {
                        let (child, value, mut check_needed) = child.remove_recursively(value);
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
                        let (child, value, mut check_needed) = child.remove_recursively(value);
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

    fn check_imbalance_after_delete(&mut self, side: Side) -> bool {
        let is_red = self.color == Color::Red;
        let sibling = self.another_child(side).as_mut().unwrap();
        let same_side_nephew_is_red = !Self::is_black(sibling.child(side));
        let another_side_nephew_is_red = !Self::is_black(sibling.another_child(side));
        if sibling.color == Color::Red {
            self.d3(side);
            false
        } else if another_side_nephew_is_red {
            self.d6(side);
            false
        } else if same_side_nephew_is_red {
            self.d5(side);
            false
        } else if is_red {
            self.d4(side);
            false
        } else {
            sibling.color = Color::Red;
            true
        }
    }

    fn d3(&mut self, side: Side) {
        self.rotate_to(side);
        self.color = Color::Black;
        let previous_self = self.child(side).as_mut().unwrap();
        previous_self.color = Color::Red;
        let sibling = previous_self.child(side.other()).as_mut().unwrap();
        if Self::is_red(sibling.child(side.other())) {
            previous_self.d6(side);
        } else if Self::is_red(sibling.child(side)) {
            previous_self.d5(side);
        } else {
            previous_self.d4(side);
        }
    }

    fn d4(&mut self, side: Side) {
        self.color = Color::Black;
        self.child(side.other()).as_mut().unwrap().color = Color::Red;
    }

    fn d6(&mut self, side: Side) {
        let color = self.color;
        self.color = Color::Black;
        self.rotate_to(side);
        self.color = color;
        self.child(side.other()).as_mut().unwrap().color = Color::Black;
    }

    fn d5(&mut self, side: Side) {
        let sibling = self.child(side.other()).as_mut().unwrap();
        sibling.color = Color::Red;
        sibling.rotate_to(side.other());
        sibling.color = Color::Black;
        self.d6(side);
    }

    fn is_black(node: &Tree<T>) -> bool {
        !Self::is_red(node)
    }
    fn is_red(node: &Tree<T>) -> bool {
        node.as_ref().is_some_and(|x| { x.color == Color::Red })
    }
}


fn new_node<T: Ord>(value: T, color: Color) -> Tree<T> {
    Some(Box::new(Node {
        value,
        left_child: None,
        right_child: None,
        color,
    }))
}

impl<T: Ord + Debug> RedBlack<T> {
    pub fn new() -> Self {
        return RedBlack { root: None };
    }

    pub fn remove(&mut self, value: &T) -> Option<T> {
        match self.root.take() {
            None => { None }
            Some(node) => {
                let returned_value;
                (self.root, returned_value) = node.remove(value);
                returned_value
            }
        }
    }

    pub fn insert(&mut self, value: T) {
        match &mut self.root {
            None => {
                self.root = new_node(value, Color::Black);
            }
            Some(node) => {
                node.insert(value);
            }
        }
    }

    pub fn find(&self, value: &T) -> Option<&T> {
        match &self.root {
            None => {
                None
            }
            Some(node) => {
                node.find(value)
            }
        }
    }

    pub fn min(&self) -> Option<&T> {
        match &self.root {
            None => {
                None
            }
            Some(node) => {
                Some(node.min())
            }
        }
    }

    pub fn max(&self) -> Option<&T> {
        match &self.root {
            None => {
                None
            }
            Some(node) => {
                Some(node.max())
            }
        }
    }

    pub fn next(&self, value: &T) -> Option<&T> {
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


#[cfg(test)]
mod tests {
    use std::fmt::Display;
    use super::*;

    #[test]
    fn test_inserting_and_deleting_keeps_tree_balanced() {
        let mut vec: Vec<u32> = (0..1000).collect();
        let mut tree = RedBlack::new();
        for (i, j) in vec.iter().enumerate() {
            tree.insert(j.clone());
            check_tree(tree.root.as_ref().unwrap(), (i + 1) as u32);
        }
        for j in vec.iter() {
            assert_eq!(Some(j), tree.find(j));
        }
        assert!(tree.root.is_some());
        let mut size = vec.len();
        for j in vec.iter() {
            assert_eq!(Some(j.clone()), tree.remove(j));
            size -= 1;
            if size > 0 {
                check_tree(tree.root.as_ref().unwrap(), size as u32);
            } else {
                assert!(tree.root.is_none());
            }
        }
    }

    fn check_tree<T: Ord + Display>(tree: &Box<Node<T>>, expected_size: u32) {
        assert_eq!(tree.color, Color::Black);
        let (_, size) = check_tree_recursively(tree);
        assert_eq!(size, expected_size);
    }

    fn check_tree_recursively<T: Ord + Display>(tree: &Box<Node<T>>) -> (u32, u32) {
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

        return if tree.color == Color::Black {
            (left + 1, right_children + left_children + 1)
        } else {
            assert_eq!(false, tree.left_child.as_ref().is_some_and(|x| { x.color == Color::Red }));
            assert_eq!(false, tree.right_child.as_ref().is_some_and(|x| { x.color == Color::Red }));
            (left, right_children + left_children + 1)
        };
    }
}