module s {
  pub enum BST[T] {
    Empty,
    Node(value: T, left: BST[T], right: BST[T]),
  }

  pub fn make[T](v: T, l: BST[T], r: BST[T]) -> BST[T] {
    return Node(value: v, left: l, right: r);
  }

  pub fn run() -> Int { return 1; }
}

use s.run;
fn main() -> Int { return run(); }
