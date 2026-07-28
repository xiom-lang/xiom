// M32-E13: Deep pattern match with nested enum payload
enum Inner { None, SomeVal(v: Int) }
enum Outer { Empty, Data(i: Inner) }
fn extract(o: Outer) -> Int {
  match o {
    Empty => { return -1; }
    Data(i) => {
      match i {
        None => { return 0; }
        SomeVal(v) => { return v; }
      }
    }
  }
}
fn main() -> Int {
  if extract(Outer.Empty) != -1 { return 1; }
  if extract(Outer.Data(Inner.None)) != 0 { return 2; }
  if extract(Outer.Data(Inner.SomeVal(99))) != 99 { return 3; }
  return 0;
}
