// M32: Struct with Int32 field mutated through Int
type Block = { addr: Int32; len: Int16; }
fn main() -> Int {
  var b: Block = Block{ addr: 300000000; len: 1000 as Int16; };
  var a: Int = b.addr as Int;
  if a == 300000000 { return 0; }
  return 1;
}
