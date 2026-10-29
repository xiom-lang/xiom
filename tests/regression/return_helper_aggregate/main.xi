// REL-20261029-2110-packages (u4): the reassembly return is
// `_ok_outcome(done)` where `done.payload` comes from a FIELD projection of an
// aggregate (`taken.payload`). The helper call is a real return, but the
// prepass only recognized `return Ok(x)` directly; `done` stayed in the drop
// set and the caller's payload was freed (UDP reassembly delivered corrupted
// bytes; the payload was already wrong at rl_on_packet).
module return_helper_aggregate

use xiom.io;
use xiom.convert;

type Entry = { payload: Vec[UInt8]; id: Int; }
type Pay = { data: Vec[UInt8]; }
type Wrap = { payload: Pay; id: Int; }
type Err = { code: Int; }

// The transport package confines Ok/Err construction to leaf helpers; this is
// the exact `_ok_outcome(done)` shape.
fn ok_wrap(w: Wrap) -> Result[Wrap, Err] {
  return Ok(w);
}

fn make_wrap() -> Result[Wrap, Err] {
  var d = Vec[UInt8].new();
  var k = 0;
  while k < 1500 {
    d.push((k % 251) as UInt8);
    k = k + 1;
  }
  var e = Entry{ payload: d, id: 7 };
  // Field projection, like `done.payload = taken.payload`.
  var w = Wrap{ payload: Pay{ data: e.payload }, id: e.id };
  return ok_wrap(w);
}

fn churn() -> Int {
  var junk = Vec[Vec[UInt8]].new();
  var i = 0;
  while i < 2000 {
    var v = Vec[UInt8].new();
    var j = 0;
    while j < 1500 {
      v.push(85);
      j = j + 1;
    }
    junk.push(v);
    i = i + 1;
  }
  return junk.len();
}

fn main() -> Int {
  let r = make_wrap();
  if r.is_err { return 1; }
  let w = r.value;
  let _j = churn();
  if w.id != 7 { return 2; }
  if w.payload.data.len() != 1500 { return 3; }
  if w.payload.data[0] != 0 as UInt8 { return 4; }
  if w.payload.data[1] != 1 as UInt8 { return 5; }
  if w.payload.data[1499] != (1499 % 251) as UInt8 { return 6; }
  io.println("return helper aggregate ok");
  return 0;
}
