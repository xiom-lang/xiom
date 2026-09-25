// m140 (packages relay #2): references to BOXED Result payload fields.
// `Result[Vec[UInt8], Str].value` holds a handle to the heap box; `&r.value`
// used to yield the handle SLOT's address, so a `&Vec` callee read the handle
// bits as data (length 0). The reference must be the POINTEE (the real Vec),
// which also makes `&mut r.value` mutation reach the box.
module m140_result_payload_ref;

pub type Holder = { v: Vec[UInt8]; }

fn take(v: &Vec[UInt8]) -> Int { return v.len(); }
fn push_more(v: &mut Vec[UInt8]) { v.push(9u8); }

fn make() -> Result[Vec[UInt8], Str] {
  var v = Vec[UInt8].new();
  v.push(1u8);
  v.push(2u8);
  v.push(3u8);
  return Ok(v);
}

fn main() -> Int {
  var r = make();
  if !r.is_ok() { return 9; }

  // Packages' minimal shape: &r.value must read the real payload.
  if take(&r.value) != 3 { return 1; }

  // The local-binding workaround keeps working.
  let v = r.value;
  if take(&v) != 3 { return 2; }

  // Mutation through the payload reference reaches the box.
  push_more(&mut r.value);
  if take(&r.value) != 4 { return 3; }

  // Plain struct field references keep working (regression guard).
  var h = Holder{ v: Vec[UInt8].new() };
  h.v.push(7u8);
  if take(&h.v) != 1 { return 4; }

  return 0;
}
