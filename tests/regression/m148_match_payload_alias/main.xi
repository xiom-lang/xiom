// m148 (benchmark relay R-2): match bindings on a persistent Option[T] must
// ALIAS the boxed payload for aggregate payloads, not copy it. Before:
// `match o { Some(v) => { v.n = 6; } }` mutated a stack copy and the
// re-read saw the old value; `Some(v) => v.push(x)` on a Vec payload kept
// len 0. The binding is registered with the pointer-backed struct-local
// convention (register = payload address, ty = struct/Vec), so field
// writes, method receivers and pushes target the payload in the box.
// Scalar payload reassignment (`Some(v) => v = 6`) stays a rebind, and
// `Some(c)` still boxes a COPY (mutating the payload does not touch `c`).
module m148_match_payload_alias;

pub type Cell = { n: Int; }

fn main() -> Int {
  // Struct payload: field mutation through the match binding persists.
  var c = Cell{ n: 5; };
  var o = Some(c);
  match o {
    Some(v) => { v.n = 6; }
    None => { }
  }
  match o {
    Some(w) => { if w.n != 6 { return 2; } }
    None => { return 3; }
  }
  // The source struct passed INTO Some was copied; it is untouched.
  if c.n != 5 { return 4; }

  // Vec payload: push through the binding persists in the payload.
  var vo = Some(Vec[Int].new());
  match vo {
    Some(v) => { v.push(7); }
    None => { }
  }
  match vo {
    Some(w) => {
      if w.len() != 1 { return 5; }
      if w[0] != 7 { return 6; }
    }
    None => { return 7; }
  }

  // Read-only match on a temporary: unchanged.
  match Some(Cell{ n: 9; }) {
    Some(t) => { if t.n != 9 { return 8; } }
    None => { return 9; }
  }

  // Option[Int] scalar rebind inside the arm does not leak out.
  var s = Some(5);
  match s {
    Some(v) => { v = 6; }
    None => { }
  }
  match s {
    Some(w) => { if w != 5 { return 10; } }
    None => { return 11; }
  }
  return 0;
}
