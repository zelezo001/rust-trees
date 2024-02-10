# Rust trees

## Introduction

Goal of crate is to implement Red-black and AVL tree in rust without using unsafe.

### Benchmarking

For purpose of benchmarking repository contain `benchmark.rs` in example folder.
This benchmark should test how long does it take to insert/delete/find sequence of numbers in benchmarked
implementation.

## Implementation

Trees are implemented by two main parts `Tree` and `Node`. `Tree` serves as a wrapper around `Node` and its only purpose
is to allow empty trees. `Node` represent actual binary search tree. BST is implemented as map and every node has two
variables key and value. Key must be comparable.

### Common structs and methods

Read operations are same for both implementations and are present in `lib.rs` on base type `rust_trees::Node`, which
contain all necessary fields for representation of BST.

For read operations `lib.rs` also provides `rust_trees::Tree` wrapping methods.

Because read operations does not mutate tree and don't need any form of backtracking, they should be implemented as
loops and not recursion.

#### Find

`find(key)` is method on `rust_trees::Node` which returns value for given key, if the tree contains such node,
otherwise `None`. As tree is BST standard binary search is used:

#### Max

`max()` is method on `rust_trees::Node` which returns borrowed `(key, value)` with the greatest key in the tree.
Program walks tree from the root and goes to right child if it can, otherwise return value of current node.

#### Min

`min()` is method on `rust_trees::Node` which returns `(key, value)` from node with the lowest key in the tree, it works
same as `max` method, just program walks to the left.

#### Next

`next(key)` is method on `rust_trees::Node` which returns (key, value) of next node by key in inorder succesion if such
node exists.
Next works in same way as search with addition of keeping record of node with the lowest encountered key larger than
given key (current_next)
If node with given key is not found or does not have right child, current_next is returned, otherwise minimal node from
right child is returned.

### AVL tree

Read operations are implemented by base struct.
Write operations and related balancing is implemented `rust_trees:avl::Node` and public API is provided
by `rust_trees:avl::Tree`. Every node keeps difference between heights of right and left child, in AVL tree difference
must be less than two.

#### Insert

`insert(key, value)` on  `rust_trees:avl::Node` walks the tree same way as search until node with same key is found or
there is no node to continue to. If node with given key exists, key and value in node are replaced and insertion ends as
no new node has been added, otherwise new node is inserted, which results in change of height, which must be updated
iteratively to the root. During this balancing may be needed (see balancing).

#### Remove

`remove(key)` on  `rust_trees:avl::Node` walks the tree same way as search until node with same key is found, if no such
node exists, `None` is returned. If node is found its popped from tree and if needed to be replaced with appropriate
child. As
removing node can result in change of height, balance factors must be updated iteratively to the root. During this
balancing may be needed (see balancing).

#### Balancing

During alteration of the tree AVL properties can be temporary broken as balance factor can be two.
Properties can be restored by performing one of these rotations:

1. right left rotation - right-heavy tree with left-leaning right child
   ```
     a               c
    / \             / \
   W   b           a   b
      / \    =>   / \ / \
     c   X       X  Y Z  W
    / \
   Y   Z
   ```
2. left right rotation - right-heavy tree with left-leaning right child
    ```
        a              c
       / \            / \
      b   W          b   a
     / \      =>    / \ / \
    X   c          X  Y Z  W
       / \
      Y   Z
   ```
3. left rotation - right-heavy tree with balanced or right-leaning right child
    ```
      a                b
     / \              / \
    X   b            a   Y
       / \    =>    / \
      Z   Y        X  Z
   ```
4. right rotation - left-heavy tree with balanced or left-leaning left child
    ```
        a            b
       / \          / \
      b   X        Z   a
     / \      =>      / \
    Z   Y            Y   X
   ```

In most cases rotations absorb height change, only exception is during removal when balance factor is zero after
rotation.

### Red-black tree

Read operations are implemented by base struct. Write operations and related balancing is
implemented `rust_trees:rb::Node` and public API is provided
by `rust_trees:rb::Tree`.

Red-back tree has the following properties:

1. Every node is either red or black.
2. A red node does not have red child.
3. None children are considered black.
4. Every path from root to leaf has same amount of black nodes.
5. Root is black.

#### Insert

walks the tree same way search does until node with same key is found or there is no node to continue to.
If node with given key exists, key and value in node are replaced and insertion ends, otherwise new red node is
inserted.
Adding red node can violate rule 2. One of following scenarios can occur:

1. Parent color is black.
2. New node is root.
3. Parent is red.

Cases 1 and 2 does not violate properties of red-black tree, so no balancing is needed.
Case 3 violates rule 2. From properties of tree grandparent is definitely black and present. (root is always black, so
parent cannot be root).
WLOG parent is left child of grandparent, then for N is inserted node tree looks:

1. N is same side as P
   ```
       G   
      / \  
     P   U 
    / \     
   N   S   
   ```
2. N is not same side as P
   ```
       G   
      / \  
     P   U 
    / \     
   S   N   
   ```

If uncle is black and tree is in 1. case, then we can achieve balance by rotation. Then after painting parent black and
grandparent red we will fix violation of rule 2.

   ```
       G            P     
      / \          / \   
     P   U   =>   N   G   
    / \              / \ 
   N   S            S   U
   ```

2&period; case can be transformed to 1. case by rotation(Note that this rotation does not break any red-black tree rule)

   ```
           G                G     
          / \              / \   
         P   U   =>       N   U   
        / \              / \     
       S   N            P   Z    
          / \          / \                       
         Y   Z        S   Y
   ```

If uncle is red, then we cannot perform rotation, but as parent is also red we can paint them both black and paint
grandparent red. This fixes violation of rule 2 between parent and inserted node but can introduce violation of the same
rule between grandparent and his parent, so we need to check it and do same steps to repair balance between them.
Eventually grandparent will be root and can be painted black without any rule violation.

#### Remove

`remove(key)` on  `rust_trees:rb::Node` walks the tree same way search does until node with same key is found, if no
such node exists, `None` is returned. If node is found its popped from tree and if needed to be replaced with
appropriate child. Popping node may result in violation of tree properties so tree needs to be updated according to in
which node was.

1. Node had two children.
2. Node has only one child.
3. Node is the root.
4. Node has no children and was red.
5. Node has no children and was black.

In case 1, we can replace popped node with the smallest node S in node's right child and remove S from the tree.
Removing
S falls in cases 2-5.

In case 2, node's child is definitely red (black node would break rule 4) and popped node is black. We can replace
popped node with its child painted black. Number of black nodes is same as before.

Popping red child does not change red-black properties of the tree, so case 3 cannot violate any rule.

Case 5 violates rule 4 and tree needs to be rebalanced.

N is our popped node WLOG is also right child of its parent, then part of the tree looks like this:

   ```
       P   
      / \  
     S   N 
    / \     
   D   C   
   ```

If parent and sibling are black, then D, C cannot be nodes, and we can paint sibling red.
This fixes violation of rule 4 in subtree where parent is root. Another balancing must be done on grandparent.

If sibling is red, then D,S are definitely present in the tree and are black. We can rotate tree and paint sibling black
and
parent red.

   ```
       P            S     
      / \          / \    
     S   N   =>   D   P    
    / \              / \   
   D   C            C   N 
   ```

This did not fixed violation of rules, but as C cannot have black children (it would be violation of rule 4)
it can be resolved by following cases where parent is red.

If parent is red and sibling and his children are black, violation of rules can be easily fixed by painting parent black
and sibling red. This restores properties of tree.

If D is red, then sibling is black and properties of the tree can be restored by performing rotation and swapping colors
of parent and sibling.

   ```
       P            S     
      / \          / \    
     S   N   =>   D   P    
    / \              / \   
   D   C            C   N 
   ```

Last possible state of subtree is that D is black and and C is red. 
We can perform rotation and swap colors of sibling and D. Now we transformed three into previous case and restore balance as mentioned before.

   ```
       P             P     
      / \           / \    
     S   N   =>    C   N    
    / \           / \       
   D   C         S   B  
      / \       / \
     A   B     D   A
   ```

--------------

##### Sources

https://en.wikipedia.org/wiki/AVL_tree
https://en.wikipedia.org/wiki/Red%E2%80%93black_tree
