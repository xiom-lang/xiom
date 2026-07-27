fn id[T](x: T) -> T { return x; }
fn main() -> Int {
  var a = id[Int](42);
  if a != 42 { return 1; }
  var b = id[Bool](true);
  if b != true { return 2; }
  return 0;
}