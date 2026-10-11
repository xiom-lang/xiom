// m270 (REL-20261010-1854-pulse + REL-20261010-2122-pulse shape): a
// module's OWN struct-returning helper called unqualified from the same
// module must bind the module's definition -- never a foreign bare
// same-named fn. This is the pulse site-mode AV minimized: handle_route's
// `out_text(...)` (returns the struct HandlerOut) was hijacked to a
// Str-returning bare `out_text`, emitting `call i8* @out_text(...)` +
// `load %struct.HandlerOut, i8*` (0xC0000005 at startup).
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module mod_a

type Out = {
  code: Int;
  body: Str;
}

fn out_text(o: &Out) -> Out {
  return Out { code: o.code + 1, body: o.body };
}

pub fn handle(code: Int) -> Out {
  let o = Out { code: code, body: "x" };
  return out_text(&o);
}
