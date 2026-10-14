// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m318 lock (REL-20261013-2215-engine-2): the by-value struct move semantic,
// pinned deliberately. The borrow checker treats every NON-Copy type
// (structs, Vecs, enums, tuples) as MOVE-ONLY: passing a bare identifier
// BY VALUE consumes the local, so reusing it trips E001 "use of moved value".
// In the default mode that is a WARNING (compilation continues -- the ABI
// still passes a copy, so the value stays readable at runtime); under
// --strict it is a hard error and compilation aborts. Scalars/Str are Copy
// (reuse free); `&`/`&mut` reuse is the expressible by-ref convention (the
// engine's public-API convention); a by-value PARAMETER forwarded twice is
// read-borrowed (no move). Four faces, one fixture:
//   (1) local struct reuse by value  -> E001 expected (the pin)
//   (2) scalar reuse                 -> no E001
//   (3) by-ref reuse                 -> no E001
//   (4) by-value param forwarded     -> no E001
// borrow_e001.rs asserts the diagnostics; the values below pin the runtime
// behavior (a "moved" local still reads back its value in default mode).
module m318_move_by_value_pin;

use xiom.io;

type Pair = { a: Int; b: Int; }

fn take(p: Pair) -> Int { return p.a + p.b; }
fn take_ref(p: &Pair) -> Int { return p.a + p.b; }
fn take_num(n: Int) -> Int { return n + 1; }

fn forward_twice(p: Pair) -> Int {
  return take(p) + take(p);
}

fn main() -> Int {
  // (1) local struct reuse by value: move-only -- E001 expected on reuse.
  var s = Pair { a: 1; b: 2 };
  let x = take(s);
  let y = take(s);
  if x != 3 { return 1; }
  if y != 3 { return 2; }

  // (2) scalar reuse: Copy -- no E001.
  var n = 5;
  let a = take_num(n);
  let b = take_num(n);
  if a != 6 { return 3; }
  if b != 6 { return 4; }

  // (3) by-ref convention: reuse expressible -- no E001.
  var t = Pair { a: 3; b: 4 };
  let c = take_ref(&t);
  let d = take_ref(&t);
  if c != 7 { return 5; }
  if d != 7 { return 6; }

  // (4) by-value param forwarded twice: read-borrowed -- no E001.
  let e = forward_twice(Pair { a: 5; b: 6 });
  if e != 22 { return 7; }

  io.println("m318 move pin: x=" + to_string(x) + " y=" + to_string(y)
    + " a=" + to_string(a) + " b=" + to_string(b)
    + " c=" + to_string(c) + " d=" + to_string(d) + " e=" + to_string(e));
  return 0;
}
