fn unwrap_or[T](opt: Option[T], default: T) -> T {
  if opt.is_some { return opt.value; }
  return default;
}

fn main() -> Int {
  var x = Some(42);
  return unwrap_or(x, 0);
}
