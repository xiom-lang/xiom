module regression.m19_default_0063

interface ByteCheck {
  fn is_byte(&self) -> Bool {
    var v = value();
    return v >= 0i8 && v <= 127i8;
  }
  fn value(&self) -> Int8;
}

type Byte = { val: Int8; }

fn Byte.value(&self) -> Int8 { return val; }

fn main() -> Int {
  var b1: Byte = Byte{ val: 100i8 };
  var b2: Byte = Byte{ val: -5i8 };
  var b3: Byte = Byte{ val: 200i8 };
  if b1.is_byte() && !b2.is_byte() && !b3.is_byte() { return 0; }
  return 1;
}
