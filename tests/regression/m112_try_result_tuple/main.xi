// m112 (L3-50): `?` on a Result with a TUPLE payload.
// `let (a, b) = two()?` used to bind BOTH names to the raw boxed-tuple
// handle (a+b printed pointer arithmetic) and the `Err` propagation path
// aliased the payload too. The `?` handler now unboxes aggregate payloads
// (tuple / nested container / struct) out of the erased i64 slot.
module m112.main

use xiom.io;

fn two() -> Result[(Int, Int), Str] {
  return Ok((5, 3));
}

fn fail() -> Result[(Int, Int), Str] {
  return Err("no");
}

fn calc() -> Result[Int, Str] {
  let (a, b) = two()?;
  return Ok(a + b);
}

fn calc_err() -> Result[Int, Str] {
  let (a, b) = fail()?;
  return Ok(a + b);
}

fn main() -> Int {
  let ok = match calc() { Ok(v) => v, Err(e) => { io.println(e); return 1; } };
  if ok != 8 { return 2; }
  match calc_err() {
    Ok(v) => { io.println(to_string(v)); return 3; }
    Err(e) => { if e != "no" { return 4; } }
  }
  io.println("ok");
  return 0;
}
