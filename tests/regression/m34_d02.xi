// M34-D02: Linked list — struct with next recursive pointer
type Node = { value: Int; next: *Node; }

fn list_length(n: *Node, acc: Int) -> Int {
  if n == (0 as *Node) { return acc; }
  var nx: *Node;
  unsafe { nx = (*n).next; }
  return list_length(nx, acc + 1);
}

fn main() -> Int {
  var empty: *Node = 0 as *Node;
  if list_length(empty, 0) != 0 { return 1; }
  return 0;
}
