// M32: Struct with UInt32 field mutated through Int
fn main() -> Int {
  type Block = { addr: UInt32; len: UInt16 };
  var b: Block = Block{ addr: 3000000000, len: 1000 };
  var a: Int = b.addr as Int;
  if a == 3000000000 { return 0; }
  return 1;
}
