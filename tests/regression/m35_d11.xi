// M35-D11: Linked list — length and search via raw *Node pointers (null only)
type Node = { value: Int; next: *Node; }

fn list_len(head: *Node) -> Int {
  var len: Int = 0;
  var cur: *Node = head;
  while cur != (0 as *Node) { len = len + 1; unsafe { cur = (*cur).next; } }
  return len;
}

fn list_has(head: *Node, val: Int) -> Bool {
  var cur: *Node = head;
  while cur != (0 as *Node) {
    var v: Int; unsafe { v = (*cur).value; }
    if v == val { return true; }
    unsafe { cur = (*cur).next; }
  }
  return false;
}

fn main() -> Int {
  var empty: *Node = 0 as *Node;
  if list_len(empty) != 0 { return 1; }
  if list_has(empty, 1) { return 2; }
  return 0;
}
