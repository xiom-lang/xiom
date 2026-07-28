// M32-G11: Generic type alias
type Id = Int;
type UserId = Id;
fn id[T](x: T) -> T { return x; }
fn get_id(x: UserId) -> UserId { return id(x); }
fn main() -> Int {
  var a: UserId = get_id(55);
  if a != 55 { return 1; }
  return 0;
}
