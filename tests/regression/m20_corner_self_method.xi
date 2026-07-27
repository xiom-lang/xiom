type C = { v: Int; }
fn C.get(self) -> Int { return self.v; }
fn main() -> Int { var c = C{ v: 99 }; if c.get() != 99 { return 1; } return 0; }