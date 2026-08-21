// M34-N20: Manual eq vs derive Eq -- dual validation with nested and generic types
type Node = { val: Int; next: Int; } derive[Eq]
type Entry[T] = { key: T; tag: Int; } derive[Eq]
fn manual_eq(a: Node, b: Node) -> Bool { return a.val == b.val && a.next == b.next; }
fn main() -> Int {
  var n1 = Node{ val: 10; next: 20; };
  var n2 = Node{ val: 10; next: 20; };
  var n3 = Node{ val: 30; next: 40; };
  var e1 = Entry[Int]{ key: 5; tag: 0; };
  var e2 = Entry[Int]{ key: 5; tag: 0; };
  if manual_eq(n1, n2) && n1 == n2 && n1 != n3 && e1 == e2 { return 0; }
  return 1;
}
