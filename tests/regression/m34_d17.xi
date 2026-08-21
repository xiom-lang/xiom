// M34-D17: List reverse -- compute reversed list properties without mutation
type Node = { value: Int; next: *Node; }

fn list_sum(n: *Node) -> Int {
  if n == (unsafe { 0 as *Node }) { return 0; }
  var val: Int;
  var nx: *Node;
  unsafe { val = (*n).value; }
  unsafe { nx = (*n).next; }
  return val + list_sum(nx);
}

fn list_length(n: *Node) -> Int {
  if n == (unsafe { 0 as *Node }) { return 0; }
  var nx: *Node;
  unsafe { nx = (*n).next; }
  return 1 + list_length(nx);
}

fn get_nth_from_end(n: *Node, k: Int, len: Int) -> Int {
  if n == (unsafe { 0 as *Node }) { return 0; }
  var val: Int;
  var nx: *Node;
  unsafe { val = (*n).value; }
  unsafe { nx = (*n).next; }
  if k == len - 1 { return val; }
  return get_nth_from_end(nx, k + 1, len);
}

fn list_product(n: *Node) -> Int {
  if n == (unsafe { 0 as *Node }) { return 1; }
  var val: Int;
  var nx: *Node;
  unsafe { val = (*n).value; }
  unsafe { nx = (*n).next; }
  return val * list_product(nx);
}

fn main() -> Int {
  var empty: *Node = unsafe { 0 as *Node };
  if list_sum(empty) != 0 { return 1; }
  if list_length(empty) != 0 { return 2; }
  if list_product(empty) != 1 { return 3; }
  if get_nth_from_end(empty, 0, 0) != 0 { return 4; }
  return 0;
}
