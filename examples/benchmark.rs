use std::cmp::{max, min};
use std::time::{Duration, Instant};
use rust_trees::avl::AVL;
use rust_trees::rb::RedBlack;

fn main() {
    search_benchmark();
    insertion_benchmark();
    deletion_benchmark();
}

const TEST_NODE_COUNTS: [u64; 3] = [1000, 100000, 10000000];
const ITERATIONS: u64 = 5;

fn search_benchmark() {
    print_test_header("Search of every value");
    for count in TEST_NODE_COUNTS {
        let mut avl_tree = AVL::new();
        for i in 0..count {
            avl_tree.insert(i, i);
        }
        run_test("AVL", count, || { || { avl_search_test(count, &avl_tree) } });
        let mut rb_tree = RedBlack::new();
        for i in 0..count {
            rb_tree.insert(i, i);
        }
        run_test("RedBlack", count, || { || { rb_search_test(count, &rb_tree) } });
    }
    print_test_footer();
}

fn insertion_benchmark() {
    print_test_header("Creation of tree");
    for count in TEST_NODE_COUNTS {

        run_test("AVL", count, || { || { avl_insert_test(count) } });
        run_test("RedBlack", count, || { || { rb_insert_test(count) } });
    }
    print_test_footer();
}

fn rb_insert_test(count: u64) {
    let mut tree = RedBlack::new();
    for i in 0..count {
        tree.insert(i, i);
    }
}

fn avl_insert_test(count: u64) {
    let mut tree = AVL::new();
    for i in 0..count {
        tree.insert(i, i);
    }
}


fn rb_search_test(count: u64, tree: &RedBlack<u64, u64>) {
    for i in 0..count {
        tree.find(&i);
    }
}

fn avl_search_test(count: u64, tree: &AVL<u64, u64>) {
    for i in 0..count {
        tree.find(&i);
    }
}


fn rb_deletion_test(count: u64, mut tree: RedBlack<u64, u64>) {
    for i in 0..count {
        tree.remove(&i);
    }
}

fn avl_deletion_test(count: u64, mut tree: AVL<u64, u64>) {
    for i in (0..count) {
        tree.remove(&i);
    }
}

fn deletion_benchmark() {
    print_test_header("Deletion of tree");
    for count in TEST_NODE_COUNTS {
        run_test("AVL", count, || {
            let mut avl_tree = AVL::new();
            for i in 0..count {
                avl_tree.insert(i, i);
            }
            || { avl_deletion_test(count, avl_tree) }
        });
        run_test("RedBlack", count, || {
            let mut rb_tree = RedBlack::new();
            for i in 0..count {
                rb_tree.insert(i, i);
            }
            || {
                rb_deletion_test(count, rb_tree)
            }
        });
    }
    print_test_footer();
}

fn print_test_header(name: &str) {
    println!("--------------------------{name}---------------------------------------")
}

fn run_test<K: FnOnce() -> (), F: Fn() -> K>(name: &str, nodes: u64, action: F) {
    let (avg, min, max) = timed(action);
    println!("{:10}-nodes: {:>8} avg: {:>14}us, min: {:>14}us, max: {:>14}us", name, nodes, avg, min, max);
}

fn print_test_footer() {
    println!();
}

fn nanos(duration: Duration) -> u64 {
    duration.as_secs() * 1_000_000_000 + duration.subsec_nanos() as u64
}

#[inline]
fn timed<K: FnOnce() -> (), F: Fn() -> K>(action: F) -> (u64, u64, u64) {
    let mut max_time = 0;
    let mut min_time = u64::MAX;
    let mut sum = 0;
    for _ in 0..(ITERATIONS) {
        let test = action();
        let now = Instant::now();
        test();
        let new_now = Instant::now();
        let current = nanos(new_now.duration_since(now));
        max_time = max(current, max_time);
        min_time = min(current, min_time);
        sum += current;
    }
    return (sum / ITERATIONS, min_time, max_time);
}