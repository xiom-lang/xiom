// m117 (interface value-receiver ABI): an interface-typed parameter receiving
// an AGGREGATE argument by value is unsupported -- the ABI would need to box
// the aggregate at the call and deref it in the callee. Codegen previously
// emitted a call that silently returned wrong values (hash probe: a=5381 for
// every input, a == c); it must now fail loudly at compile time instead.
module m117.interface_value_abi

interface Hasher2 { fn write_int(self, n: Int); }
interface H2 { fn hash(self, hasher: Hasher2); }

type D2 = { state: Int; }
fn D2.new() -> D2 { return D2{ state: 1 }; }
fn D2.write_int(self, n: Int) { self.state = self.state + n; }

impl H2[Int] {
  fn hash(self, hasher: Hasher2) { hasher.write_int(self); }
}

fn main() -> Int {
  var d = D2.new();
  var a = 42;
  a.hash(d);
  return 0;
}
