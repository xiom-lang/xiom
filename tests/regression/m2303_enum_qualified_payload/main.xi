// REL-20261015-2303-packages lock: a QUALIFIED enum-variant constructor with
// named args (`LVal.LDbl(value: 7.25)`) must keep the parent enum type, and a
// Float64 payload in a field name shared across variants (erased i64 slot)
// must be bit-packed, not fptosi-truncated (7.25 used to read as 3.458e-323).
module m2303_enum_qualified_payload

use xiom.io;

enum LVal {
  LStr(value: Str),
  LDbl(value: Float64),
  LInt(value: Int),
}

fn main() -> Int {
  let a: LVal = LVal.LStr(value: "hello");
  match a {
    LVal.LStr(s) => { if s != "hello" { return 1; } },
    LVal.LDbl(_) => { return 2; },
    LVal.LInt(_) => { return 3; },
  }
  let b: LVal = LVal.LDbl(value: 7.25);
  match b {
    LVal.LDbl(d) => { if d != 7.25 { return 4; } },
    LVal.LStr(_) => { return 5; },
    LVal.LInt(_) => { return 6; },
  }
  // Qualified + UNANNOTATED (used to compile but never match any arm).
  let c = LVal.LDbl(value: 10.239);
  match c {
    LDbl(d) => { if d != 10.239 { return 7; } },
    LStr(_) => { return 8; },
    LInt(_) => { return 9; },
  }
  // Unqualified constructor, qualified match.
  let e = LDbl(value: 3.5);
  match e {
    LVal.LDbl(d) => { if d != 3.5 { return 10; } },
    LVal.LStr(_) => { return 11; },
    LVal.LInt(_) => { return 12; },
  }
  let n = LVal.LInt(value: 42);
  match n {
    LVal.LInt(v) => { if v != 42 { return 13; } },
    LVal.LStr(_) => { return 14; },
    LVal.LDbl(_) => { return 15; },
  }
  io.println("enum qualified payload ok");
  return 0;
}
