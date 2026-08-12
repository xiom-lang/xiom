module m37_from_bytes_fn
// BUG 25 #3 fix: a USER fn named `from_bytes` (or from_cstring/from_utf8)
// must not be hijacked by the compiler's Str-from-bytes builtin intercept —
// the intercept only fires when no real fn with that name is registered.

fn from_bytes(b: Vec[UInt8]) -> Int {
  return 42;
}

fn main() -> Int {
  var v = Vec[UInt8].new();
  var r = from_bytes(v);
  if r != 42 { return 1; }
  return 0;
}
