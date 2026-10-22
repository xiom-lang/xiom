// m1827 (REL-20261022-1827-packages, REL-20261022-1645-packages item A):
// a DIRECT `Err(E.Variant(payload))` pushed into a Vec must keep the enum
// payload. The erased `%struct.Result` construction used to be stored with
// the concrete monomorph element size, so the Str payload slot was garbage
// and the read-back reason was empty. The local-bound control keeps working.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m1827_err_str_vec_push

use xiom.io;

pub enum E {
  Bad(reason: Str),
}

fn err(reason: Str) -> Result[Str, E] {
  return Err(E.Bad(reason));
}

fn e_text(e: E) -> Str {
  match e {
    E.Bad(reason) => { return reason; },
  };
}

fn read_reason(r: Result[Str, E]) -> Str {
  let e: E = r.error;
  return e_text(e);
}

pub fn main() -> Int {
  var failures: Int = 0;

  var direct: Vec[Result[Str, E]] = Vec[Result[Str, E]].new();
  direct.push(Err(E.Bad("direct")));
  let rd: Result[Str, E] = direct[0];
  let ds: Str = read_reason(rd);
  if ds != "direct" { io.println("direct=" + ds); failures = failures + 1; }

  var local: Vec[Result[Str, E]] = Vec[Result[Str, E]].new();
  let r0: Result[Str, E] = err("via");
  local.push(r0);
  let rl: Result[Str, E] = local[0];
  let ls: Str = read_reason(rl);
  if ls != "via" { io.println("via=" + ls); failures = failures + 1; }

  if failures == 0 { io.println("m1827 direct Err(Str) push ok"); }
  return failures;
}
