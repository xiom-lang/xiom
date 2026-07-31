module regression.m19_default_0022

interface Sizable {
  fn is_empty(&self) -> Bool { return size() == 0; }
  fn size(&self) -> Int;
}

type Buffer = { len: Int; }

fn Buffer.is_empty(self) -> Bool { return self.size() == 0; }


fn Buffer.size(&self) -> Int { return len; }

fn main() -> Int {
  var empty: Buffer = Buffer{ len: 0 };
  var full: Buffer = Buffer{ len: 5 };
  if empty.is_empty() && !full.is_empty() { return 0; }
  return 1;
}
