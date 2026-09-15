// m75 other: an unrelated same-leaf/same-fn module with the LONGEST key.
// It exists so the order-independent fallback scan (longest-qualified first)
// still cannot guess the delegation target: only the checker-recorded
// owner-qualified binding (R20) picks the canonical module. Without it the
// shim would silently delegate here (encode(10) -> 1000 instead of 20).
module m75other.base32

pub fn encode(x: Int) -> Int {
  return x * 100;
}

pub fn name() -> Str {
  return "other";
}

pub fn decode(x: Int) -> Result[Int, Str] {
  if x < 0 {
    return Err("other-neg");
  }
  return Ok(x * 100);
}
