# Tree

Providing implementation of red-black and AVL binary search tree in rust.

## Usage

To add this library to your project add following snippet to `Cargo.toml`.

```
[dependencies]
rust_trees = { git = "https://github.com/zelezo001/rust-trees.git" }
```

### Red-black tree

```rust
// Creating empty RB tree
let mut tree = rust_trees::rb::RedBlack::new();


```
### AVL tree

```rust
// Empty tree is created by calling new function
let mut avl_tree = rust_trees::avl::AVL::new();

```

## Benchmarks

To run prepared benchmark comparing performance of trees run:

```
cargo run --release --example benchmark
```