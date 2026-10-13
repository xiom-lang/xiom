// m294 lock (REL-20261010-1549-packages-9): a bare `&mut Int` read in ANY
// value sink must yield the POINTEE, never the address. m197 covered
// `return p` + arithmetic (m197_mut_ref_bare_read); this lock covers the
// remaining sinks: untyped/typed bindings, plain assignment, `*q = p`
// store-through, Vec index read/write operands, scalar cast, unary
// negation. Exit = failure count (0 = green).
// Pure XIOM. Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module m294_mut_int_bare

use xiom.io;

fn read_bare(p: &mut Int) -> Int {
  return p;
}

fn bind_bare(p: &mut Int) -> Int {
  let y = p;
  return y;
}

fn typed_bind(p: &mut Int) -> Int {
  let y: Int = p;
  return y;
}

fn assign_bare(p: &mut Int) -> Int {
  var w = 0;
  w = p;
  return w;
}

fn deref_write_bare(q: &mut Int, p: &mut Int) -> Int {
  *q = p;
  return *q;
}

fn index_read(p: &mut Int) -> Int {
  var v = [10, 20, 30];
  return v[p];
}

fn index_write(p: &mut Int) -> Int {
  var v = [0, 0, 0];
  v[1] = p;
  return v[1];
}

fn cast_bare(p: &mut Int) -> Int {
  return p as Int;
}

fn negate_bare(p: &mut Int) -> Int {
  return -p;
}

fn arith_bare(p: &mut Int) -> Int {
  return p + 1;
}

fn use_val(v: Int) -> Int {
  return v * 2;
}

fn pass_bare(p: &mut Int) -> Int {
  return use_val(p);
}

fn compare_bare(p: &mut Int) -> Int {
  if p == 41 { return 1; }
  return 0;
}

fn main() -> Int {
  var bad = 0;
  var x = 41;

  if read_bare(x) != 41 { bad = bad + 1; }
  if bind_bare(x) != 41 { bad = bad + 1; }
  if typed_bind(x) != 41 { bad = bad + 1; }
  if assign_bare(x) != 41 { bad = bad + 1; }

  var q = 0;
  if deref_write_bare(q, x) != 41 { bad = bad + 1; }

  var ix = 1;
  if index_read(ix) != 20 { bad = bad + 1; }
  if index_write(x) != 41 { bad = bad + 1; }

  if cast_bare(x) != 41 { bad = bad + 1; }
  if negate_bare(x) != -41 { bad = bad + 1; }
  if arith_bare(x) != 42 { bad = bad + 1; }
  if pass_bare(x) != 82 { bad = bad + 1; }
  if compare_bare(x) != 1 { bad = bad + 1; }

  io.println("m294 failures=" + to_string(bad));
  return bad;
}
