module m21_struct_mut_024
type Node = { value: Int; next: Option[Int]; }
fn main() -> Int {
  var n: Node = Node{ value: 1; next: None; };
  n.next = Some(99);
  match n.next {
    Some(v) => if v == 99 { return 0; },
    None => return 1,
  }
  return 1;
}
