// m320 lock (REL-20261013-2035-engine): the `&Str` PARAMETER spelling.
// Pre-fix, `s: &Str` warned `unknown type '&Str' -- defaulting to i64` and
// the value-use path bitcast the slot ADDRESS (i8**) to i8*, so str_slice
// read the stack bytes as the string (copy_ref printed heap garbage while
// the by-value control printed `hell`). The by-ref param must auto-deref to
// the Str handle like plain Str; `&mut Str` params and plain-Str controls
// pin the boundary. Distinct exit codes per shape; green = 0.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module m320_ref_str_param;

use xiom.string;
use xiom.io;

fn copy_ref(s: &Str) -> Str {
  var out = "";
  var i = 0;
  while i < 4 { out = out + str_slice(s, i, i + 1); i = i + 1; }
  return out;
}

fn copy_mut_ref(s: &mut Str) -> Str {
  var out = "";
  var i = 0;
  while i < 4 { out = out + str_slice(s, i, i + 1); i = i + 1; }
  return out;
}

fn copy_val(s: Str) -> Str {
  var out = "";
  var i = 0;
  while i < 4 { out = out + str_slice(s, i, i + 1); i = i + 1; }
  return out;
}

fn ref_len(s: &Str) -> Int { return str_len(s); }
fn chain_ref(s: &Str) -> Int { return ref_len(s); }

fn main() -> Int {
  var a = "hello";
  let r1 = copy_ref(a);
  if r1 != "hell" { io.println("copy_ref=" + r1); return 1; }

  let r2 = copy_val(a);
  if r2 != "hell" { io.println("copy_val=" + r2); return 2; }

  var b = "world";
  let r3 = copy_mut_ref(&mut b);
  if r3 != "worl" { io.println("copy_mut_ref=" + r3); return 3; }

  if ref_len(a) != 5 { return 4; }
  if chain_ref(a) != 5 { return 5; }

  io.println("m320 ref-str-param ok");
  return 0;
}
