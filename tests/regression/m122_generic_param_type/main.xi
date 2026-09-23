// m122 (R69): a monomorphised generic param kept a STALE global XIOM type.
// `local_xiom_types` is global across functions and the mono param path never
// recorded plain params, so `fn show[T](x: T) -> Str { return x.to_str(); }`
// inherited `x: Float64` from an earlier emitted function (xiom.fmt), and
// show(99) printed 4.891e-322 (the i64 bits read as a double) while show("hi")
// printed 0. The mono path now mirrors compile_fn and records the substituted
// param type (T=Int -> "Int"), so the to_str sugar picks the right conversion.
module m122_generic_param_type

fn show[T](x: T) -> Str {
  return x.to_str();
}

fn pair_show[A, B](a: A, b: B) -> Str {
  return a.to_str() + ":" + b.to_str();
}

fn main() -> Int {
  if show(99) != "99" { return 1; }
  if show("hi") != "hi" { return 2; }
  if show(true) != "true" { return 3; }
  if show(2.5) != "2.5" { return 4; }
  if show(7 as UInt) != "7" { return 5; }
  if pair_show(1, "x") != "1:x" { return 6; }
  if pair_show("y", 2) != "y:2" { return 7; }
  if pair_show(true, 2.5) != "true:2.5" { return 8; }
  return 0;
}
