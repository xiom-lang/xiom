// m38_round8_catalog_mut_self -- round-8 (2026-08-20) regression:
// catalog &mut self receiver wiring for user generic structs --
// VecDeque/Stack/Queue/LinkedList/BTreeMap mutations were entirely LOST
// through the catalog (calls hijacked same-leaf methods of other types
// like Reverse.push_front because the non-pub generic type decls +
// methods were never injected). Same code passed user-space (vd3-vd5).
module m38_round8_catalog_mut_self
use xiom.collections;

fn main() -> Int {
  // 1. VecDeque mutations through the catalog (round-8 vd2/vd6 family).
  var dq = VecDeque[Int].new();
  dq.push_front(20);
  if dq.len() != 1 { return 1; }
  dq.push_back(30);
  if dq.len() != 2 { return 2; }
  match dq.pop_front() {
    Some(x) => { if x != 20 { return 3; } }
    None => { return 4; }
  }
  if dq.len() != 1 { return 5; }

  // 2. Stack (push/pop via &mut self).
  var st = Stack[Int].new();
  st.push(5);
  st.push(6);
  match st.pop() {
    Some(x) => { if x != 6 { return 6; } }
    None => { return 7; }
  }

  // 3. Queue (enqueue/dequeue).
  var q = Queue[Int].new();
  q.enqueue(1);
  q.enqueue(2);
  match q.dequeue() {
    Some(x) => { if x != 1 { return 8; } }
    None => { return 9; }
  }

  // 4. LinkedList (push_front/pop_back).
  var ll = LinkedList[Int].new();
  ll.push_front(10);
  ll.push_front(11);
  match ll.pop_back() {
    Some(x) => { if x != 10 { return 10; } }
    None => { return 11; }
  }

  // 5. BTreeMap insert/get (non-pub generic family).
  var m = BTreeMap[Int, Str].new();
  m.insert(3, "c");
  m.insert(1, "a");
  m.insert(2, "b");
  match m.get(2) {
    Some(v) => { if v != "b" { return 12; } }
    None => { return 13; }
  }
  return 0;
}
