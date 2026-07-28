// M35-L25: Stack allocation pattern — multiple struct stack allocations with interleaved access
type Frame = { id: Int; next_id: Int; data: Int; }

fn main() -> Int {
  var f1 = Frame{ id: 1; next_id: 2; data: 10; };
  var f2 = Frame{ id: 2; next_id: 3; data: 20; };
  var f3 = Frame{ id: 3; next_id: 0; data: 30; };
  var sum: Int = f1.data + f2.data + f3.data;
  if sum != 60 { return 1; }
  if f1.id != 1 { return 2; }
  if f2.next_id != 3 { return 3; }
  if f3.next_id != 0 { return 4; }
  return 0;
}
