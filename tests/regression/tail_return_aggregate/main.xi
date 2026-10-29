// REL-20261028-1641 follow-up: a function body whose trailing expression is
// `Ok(conn)` / `Some(conn)` (implicit return, no `return` keyword) transfers
// the aggregate to the caller. The prepass used to scan that tail as a plain
// escaping expression, so the aggregate exit drop freed the Vec buffer the
// caller now owns -> use-after-free once the block is reused
// (p_wave162 row 12; stdlib tools/known_failures/p_borrowed_mut_grown_vec_uaf.xi).
module tail_return_aggregate

use xiom.io;
use xiom.convert;

type S = { buf: Vec[UInt8]; }

fn fill(conn: &mut S) -> Int {
  var i = 0;
  while i < 18 {
    conn.buf.push(100);
    i = i + 1;
  }
  return 18;
}

fn read_head(conn: &mut S) -> Result[Str, Str] {
  var guard = 0;
  while guard < 4 {
    if conn.buf.len() > 10 {
      return Ok("head");
    }
    let _n = fill(conn);
    guard = guard + 1;
  }
  return Err("not terminated");
}

// Implicit tail return through Result.
fn connect_result() -> Result[S, Str] {
  var conn = S{ buf: Vec[UInt8].new() };
  match read_head(&mut conn) {
    Ok(_) => {},
    Err(e) => { return Err(e); },
  }
  Ok(conn)
}

// Implicit tail return through Option.
fn connect_option() -> Option[S] {
  var conn = S{ buf: Vec[UInt8].new() };
  let _n = fill(&mut conn);
  Some(conn)
}

fn churn() -> Int {
  var junk: Vec[UInt8] = Vec[UInt8].new();
  var k = 0;
  while k < 40000 {
    junk.push(85);
    k = k + 1;
  }
  return junk.len();
}

fn main() -> Int {
  var round = 0;
  while round < 20 {
    let r = connect_result();
    if r.is_err { return 1; }
    let c = r.value;
    let _j = churn();
    if c.buf.len() != 18 { return 2; }
    if c.buf[17] != 100 { return 3; }

    match connect_option() {
      Some(o) => {
        let _j2 = churn();
        if o.buf.len() != 18 { return 4; }
        if o.buf[0] != 100 { return 5; }
      }
      None => { return 6; }
    }
    round = round + 1;
  }
  io.println("tail return aggregate ok rounds=" + convert.int_to_string(round));
  return 0;
}
