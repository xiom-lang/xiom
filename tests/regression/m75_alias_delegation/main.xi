// m75: R20 same-leaf alias delegation through the catalog. The shim module
// (m75conv.base32) and the canonical module (m75canon.base32) share the leaf
// "base32"; the shim's bodies call the canonical fn through an alias
// (`enc32.*`). Values must flow through the delegation exactly:
//   - Int-returning leg:      base32.encode(10) == 20
//   - Str-returning leg:      base32.name() == "canon"
//   - Result Ok leg:          base32.decode(4)  == Ok(8)
//   - Result Err leg:         base32.decode(-3) == Err("neg")
module m75main

// NOTE: m75other (a same-leaf/same-fn module with the LONGEST qualified key)
// is imported too, so the delegation target cannot be guessed by any
// name-suffix heuristic: only the checker-recorded owner-qualified binding
// picks m75canon. Its fns are never called.
use m75other.base32 as other;
use m75conv.base32;

fn check_ok(x: Int) -> Bool {
  match base32.decode(x) {
    Ok(v) => { return v == x * 2; },
    Err(e) => { return false; },
  }
}

fn check_err(x: Int) -> Bool {
  match base32.decode(x) {
    Ok(v) => { return false; },
    Err(e) => { return e == "neg"; },
  }
}

fn main() -> Int {
  if base32.encode(10) != 20 { return 1; }
  if base32.name() != "canon" { return 2; }
  if !check_ok(4) { return 3; }
  if !check_err(-3) { return 4; }
  return 0;
}
