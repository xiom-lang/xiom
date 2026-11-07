// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// REL-20261104-1820-playground: matching a tuple of containers lost every
// binding (`(Some(a), Ok(b))` printed 0 / empty on v0.65.5/v0.65.6).
// The tuple check block matched unconditionally and the arm-body destructure
// only bound top-level Ident elements, so nested Option/Result payloads were
// never checked or bound. This fixture locks the four payload shapes plus a
// wildcard arm and the plain tuple / let-destructure controls.
module rel20261104_1820_tuple_option_result

use xiom.io;

fn describe(opt: Option[Int], res: Result[Int, Str]) -> Int {
  match (opt, res) {
    (Some(a), Ok(b)) => {
      io.println("some_ok a=" + to_string(a) + " b=" + to_string(b));
      return 1;
    }
    (None, Err(e)) => {
      io.println("none_err e=" + e);
      return 2;
    }
    (Some(a), Err(e)) => {
      io.println("some_err a=" + to_string(a) + " e=" + e);
      return 3;
    }
    (None, Ok(b)) => {
      io.println("none_ok b=" + to_string(b));
      return 4;
    }
    _ => {
      io.println("unreachable_pair");
      return 5;
    }
  }
}

fn only_some_ok(opt: Option[Int], res: Result[Int, Str]) -> Int {
  match (opt, res) {
    (Some(a), Ok(b)) => {
      io.println("matched a=" + to_string(a) + " b=" + to_string(b));
      return 1;
    }
    _ => {
      io.println("wildcard");
      return 2;
    }
  }
}

fn main() -> Int {
  if describe(Some(42), Ok(7)) != 1 { return 10; }
  if describe(None, Err("boom")) != 2 { return 11; }
  if describe(Some(5), Err("bad")) != 3 { return 12; }
  if describe(None, Ok(9)) != 4 { return 13; }

  if only_some_ok(Some(3), Ok(8)) != 1 { return 20; }
  if only_some_ok(None, Err("nope")) != 2 { return 21; }

  // Direct value checks: the old lowering bound every payload to 0 / NULL,
  // so arm selection alone is not enough to catch the regression.
  match (Some(42), Ok(7)) {
    (Some(a), Ok(b)) => {
      if a != 42 { return 30; }
      if b != 7 { return 31; }
    }
    _ => { return 32; }
  }
  match (None, Err("boom")) {
    (None, Err(e)) => {
      if e != "boom" { return 33; }
    }
    _ => { return 34; }
  }
  match (Some(5), Err("bad")) {
    (Some(a), Err(e)) => {
      if a != 5 { return 35; }
      if e != "bad" { return 36; }
    }
    _ => { return 37; }
  }
  match (None, Ok(9)) {
    (None, Ok(b)) => {
      if b != 9 { return 38; }
    }
    _ => { return 39; }
  }

  let pair = (11, 22);
  match pair {
    (x, y) => {
      if x != 11 { return 40; }
      if y != 22 { return 41; }
      io.println("pair x=" + to_string(x) + " y=" + to_string(y));
    }
  }

  let (p, q) = (33, 44);
  if p != 33 { return 42; }
  if q != 44 { return 43; }
  io.println("let p=" + to_string(p) + " q=" + to_string(q));
  return 0;
}
