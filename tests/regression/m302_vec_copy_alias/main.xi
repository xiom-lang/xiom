// m302 (REL-20261011-1659-orbitdb): a Vec bound from an ALIASING RHS must
// own its storage. Pre-fix, `var copy = src; copy[0] = 99;` left `copy`
// sharing `src`'s buffer (`src[0]` read 99) while push/pop only LOOKED like
// copy-on-write (per-handle len + realloc on growth). The binding now takes a
// deep copy of the payload for Ident/Index/Field RHS shapes; fresh values
// (`Vec.new()`, `.clone()`, call returns) stay uncopied, and push/pop are
// unchanged. The owned-rebuild block is the ORBITDB workaround (probe_clean)
// kept as a control.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module m302_vec_copy_alias;

use xiom.io;
use xiom.convert;

type Holder = {
  keys: Vec[Int];
  tag: Int;
}

fn main() -> Int {
  var fails: Int = 0;

  // A: flat copy -- element assignment through the copy must not write
  // through to the source (src[0] stays 1, src.len stays 2).
  var src = Vec[Int].new();
  src.push(1);
  src.push(2);
  var copy = src;
  copy[0] = 99;
  if src[0] != 1 {
    fails = fails + 1;
    io.println("  [ALIAS] A flat copy copy[0]=99 wrote through: src[0]=" + convert.int_to_string(src[0]));
  }
  if copy[0] != 99 {
    fails = fails + 1;
    io.println("  [OWN] A flat copy lost its own element: copy[0]=" + convert.int_to_string(copy[0]));
  }
  if src.len() != 2 {
    fails = fails + 1;
    io.println("  [LEN] A source len changed: src.len=" + convert.int_to_string(src.len()));
  }

  // A-control: push on the copy keeps the pre-fix behavior (per-handle
  // length; growth reallocs) -- src.len=2, copy.len=3.
  copy.push(3);
  if src.len() != 2 {
    fails = fails + 1;
    io.println("  [PUSH] A push leaked into the source: src.len=" + convert.int_to_string(src.len()));
  }
  if copy.len() != 3 {
    fails = fails + 1;
    io.println("  [PUSH] A copy len wrong: copy.len=" + convert.int_to_string(copy.len()));
  }

  // B: arena handle (Vec of Vecs) -- a handle extracted from the arena must
  // own its storage; the pinned element must keep its value.
  var arena = Vec[Vec[Int]].new();
  var a = Vec[Int].new();
  a.push(1);
  a.push(2);
  arena.push(a);
  var extracted = arena[0];
  arena.push(extracted);
  var work = arena[1];
  work[0] = 99;
  if arena[0][0] != 1 {
    fails = fails + 1;
    io.println("  [ALIAS] B arena handle wrote through: arena[0][0]=" + convert.int_to_string(arena[0][0]));
  }
  if work[0] != 99 {
    fails = fails + 1;
    io.println("  [OWN] B arena handle lost its own element: work[0]=" + convert.int_to_string(work[0]));
  }

  // B-control: pop on the local handle keeps the pre-fix behavior (the store
  // element's length is unchanged -- it was already copy-at-bind).
  work.pop();
  if work.len() != 1 {
    fails = fails + 1;
    io.println("  [POP] B local pop len wrong: work.len=" + convert.int_to_string(work.len()));
  }
  if arena[1].len() != 2 {
    fails = fails + 1;
    io.println("  [POP] B pop leaked into the store: arena[1].len=" + convert.int_to_string(arena[1].len()));
  }

  // C: struct field extract -- the extracted handle owns; the struct's
  // inline Vec keeps its value.
  var s = Holder { keys: Vec[Int].new(), tag: 0 };
  s.keys.push(1);
  var k = s.keys;
  k[0] = 99;
  if s.keys[0] != 1 {
    fails = fails + 1;
    io.println("  [ALIAS] C struct field wrote through: s.keys[0]=" + convert.int_to_string(s.keys[0]));
  }
  if k[0] != 99 {
    fails = fails + 1;
    io.println("  [OWN] C struct field copy lost its element: k[0]=" + convert.int_to_string(k[0]));
  }

  // D: assignment form (`b = a;`) -- owns after the copy.
  var da = Vec[Int].new();
  da.push(1);
  var db = Vec[Int].new();
  db.push(5);
  db = da;
  db[0] = 99;
  if da[0] != 1 {
    fails = fails + 1;
    io.println("  [ALIAS] D assignment wrote through: da[0]=" + convert.int_to_string(da[0]));
  }
  if db[0] != 99 {
    fails = fails + 1;
    io.println("  [OWN] D assignment copy lost its element: db[0]=" + convert.int_to_string(db[0]));
  }

  // E: owned-rebuild control (the ORBITDB probe_clean workaround): a
  // manually rebuilt copy owns its storage (was already correct).
  var esrc = Vec[Int].new();
  esrc.push(1);
  esrc.push(2);
  var ecopy = Vec[Int].new();
  var i = 0;
  while i < esrc.len() {
    ecopy.push(esrc[i]);
    i = i + 1;
  }
  ecopy[0] = 99;
  if esrc[0] != 1 {
    fails = fails + 1;
    io.println("  [CONTROL] E owned rebuild src changed: esrc[0]=" + convert.int_to_string(esrc[0]));
  }
  if ecopy[0] != 99 {
    fails = fails + 1;
    io.println("  [CONTROL] E owned rebuild copy wrong: ecopy[0]=" + convert.int_to_string(ecopy[0]));
  }

  io.println("m302_vec_copy_alias: " + convert.int_to_string(fails) + " failing checks");
  return fails;
}
