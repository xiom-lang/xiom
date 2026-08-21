module m37_opt_payload_value
// BUG 25 #5 regression: Option/Result `.value`/`.error` field reads must
// return the ACTUAL payload (Str/Vec/Float), not the raw i64 slot bits
// (pointer-as-number, wrong len/bit pattern). Match extraction was already
// payload-aware; the field-read path now is too.

use xiom.io;

fn main() -> Int {
  // Str payload
  var o = Some("hello");
  var v = o.value;
  if v != "hello" { return 1; }
  // Vec payload -- read, then use (push/len/index)
  var ov = Some(Vec[Int].new());
  var vec = ov.value;
  vec.push(5);
  if vec.len() != 1 { return 2; }
  if vec[0] != 5 { return 3; }
  // Int payload unaffected
  var oi = Some(42);
  if oi.value != 42 { return 4; }
  // Result .error
  var r = Err(7);
  if r.error != 7 { return 5; }
  // Float64 payload
  var of = Some(2.5);
  if of.value != 2.5 { return 6; }
  return 0;
}
