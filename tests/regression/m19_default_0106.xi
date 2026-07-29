module regression.m19_default_0106

interface Bit32Check {
  fn is_32bit(&self) -> Bool { var v = value(); return v >= -2147483648i64 && v <= 2147483647i64; }
  fn value(&self) -> Int64;
}

type Data = { x: Int64; }

fn Data.value(&self) -> Int64 { return x; }

fn main() -> Int {
  var d: Data = Data{ x: 100i64 };
  if d.value() != 100i64 { return 1; }
  if d.is_32bit() != true { return 2; }
  return 0;
}
