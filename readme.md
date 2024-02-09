# Rust trees

Rust_trees crate provides implementation of red-black and AVL tree in rust.

## Usage

To add this library to your project add following snippet to `Cargo.toml`.

```
[dependencies]
rust_trees = { git = "https://github.com/zelezo001/rust-trees.git" }
```

### Red-black tree

```rust
// Creating empty Red-black tree
let mut tree = rust_trees::rb::RedBlack::new();
// new value can be inserted into tree with insert
let key = 0; // key must implement Ord
let value = 3;

tree.insert(key, value);

// node can be removed by remove, which returns (key, value)
let node_or_none = tree.remove(key);

// value can be find by key
let borrowed_value_or_none = tree.find(key);

// next returns (key, value) with next key in inorder succession  
let borrowed_next_node_or_none = tree.next(key);

// min can be used to get (key, value) with the smallest key
let borrowed_smallest_node_or_none = tree.min();

// min can be used to get (key, value) with the largest key
let borrowed_largest_node_or_none = tree.max();

```
### AVL tree

```rust
// Creating empty AVL tree
let mut tree = rust_trees::avl::AVL::new();
// new value can be inserted into tree with insert
let key = 0; // key must implement Ord
let value = 3;

tree.insert(key, value);

// node can be removed by remove, which returns (key, value)
let node_or_none = tree.remove(key);

// value can be find by key
let borrowed_value_or_none = tree.find(key);

// next returns (key, value) with next key in inorder succession  
let borrowed_next_node_or_none = tree.next(key);

// min can be used to get (key, value) with the smallest key
let borrowed_smallest_node_or_none = tree.min();

// min can be used to get (key, value) with the largest key
let borrowed_largest_node_or_none = tree.max();

```

## Benchmarks

To run prepared benchmark comparing performance of trees run:

```
cargo run --release --example benchmark
```