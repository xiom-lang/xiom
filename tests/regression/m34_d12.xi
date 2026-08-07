// M34-D12: List append/prepend — linked list with recursive pointer and length computation
type Node = { value: Int; next: *Node; }

fn list_length(n: *Node, acc: Int) -> Int {
  if n == (unsafe { 0 as *Node }) { return acc; }
  var nx: *Node;
  unsafe { nx = (*n).next; }
  return list_length(nx, acc + 1);
}

fn list_last(n: *Node) -> *Node {
  if n == (unsafe { 0 as *Node }) { return unsafe { 0 as *Node }; }
  var nx: *Node;
  unsafe { nx = (*n).next; }
  if nx == (unsafe { 0 as *Node }) { return n; }
  return list_last(nx);
}

fn list_nth(n: *Node, k: Int) -> *Node {
  if n == (unsafe { 0 as *Node }) || k <= 0 { return n; }
  var nx: *Node;
  unsafe { nx = (*n).next; }
  return list_nth(nx, k - 1);
}

fn main() -> Int {
  var empty: *Node = unsafe { 0 as *Node };
  if list_length(empty, 0) != 0 { return 1; }
  if list_length(empty, 5) != 5 { return 2; }
  if list_last(empty) != (unsafe { 0 as *Node }) { return 3; }
  if list_nth(empty, 0) != (unsafe { 0 as *Node }) { return 4; }
  return 0;
}
