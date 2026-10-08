// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m223 (C-ORBIT-01): a match-bound Ok/Some payload behind a type ALIAS must
// keep its concrete type. `DbResult[Row]` (alias of `Result[T, DbError]`) used
// to bind the payload as a wildcard, so `decoded.get_column(0)` misresolved to
// the module fn ("'get_column' expects 2 argument(s), found 1") and 4 T001s
// blocked the ORBITDB transaction module. Alias applications must substitute
// their args (`DbResult[Row]` -> `Result[Row, DbError]`) before payload
// extraction; plain aliases and annotated lets resolve the same way.

module m223_alias_result_payload

type Cell = {
  n: Int;
  tag: Int;
}

pub fn Cell.get_n(c: &Cell) -> Int {
  return c.n;
}

// Generic alias: applied as CellRes[Cell].
pub type CellRes[T] = Result[T, Int];
// Plain (non-generic) alias.
pub type PlainRes = Result[Cell, Int];

fn make_alias() -> CellRes[Cell] { return Ok(Cell{ n: 7, tag: 1 }); }
fn make_plain() -> PlainRes { return Ok(Cell{ n: 9, tag: 2 }); }

fn via_alias() -> Int {
  let r = make_alias();
  match r {
    Err(e) => { return 10 + e; }
    Ok(x) => {
      if x.n != 7 { return 20 + x.n; }
      if x.get_n() != 7 { return 30 + x.get_n(); }
      return 0;
    }
  }
}

fn via_plain() -> Int {
  let r = make_plain();
  match r {
    Err(_) => { return 1; }
    Ok(x) => {
      if x.get_n() != 9 { return 2; }
      return 0;
    }
  }
}

fn via_annotation() -> Int {
  let r: CellRes[Cell] = Ok(Cell{ n: 4, tag: 3 });
  match r {
    Err(_) => { return 1; }
    Ok(x) => {
      if x.tag != 3 { return 3; }
      return 0;
    }
  }
}

fn main() -> Int {
  if via_alias() != 0 { return 1; }
  if via_plain() != 0 { return 2; }
  if via_annotation() != 0 { return 3; }
  return 0;
}
