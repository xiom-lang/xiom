// Same-leaf struct: `Node{ value, children }` (bench_memory shape).
module m89.types

pub type Node = {
  value: Int;
  children: Vec[Int];
}

pub fn make_node(v: Int) -> Node {
  return Node{ value: v, children: Vec[Int].new() };
}
