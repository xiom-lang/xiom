// m316 (REL-20261014-1158-engine): a struct literal whose FIRST field is
// named `spawn` (`Ship{ spawn: on; x: 7 }`) mis-parsed because the parser's
// struct-literal lookahead tested only TokenKind::Ident; `spawn` lexes as a
// soft-keyword token, so the literal was not recognized ("expected ';',
// found '{'"), the enclosing fn was dropped from the module AST, and
// consumers silently lost the export (undefined / "cannot call 'ship_ready'").
// Renaming the field is the engine's witness workaround; the name must never
// matter. Pre-fix this fixture fails: main's `veh.ship_ready(...)` calls are
// unresolved (T001 cannot call 'ship_ready' on this expression).
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module veh

type Ship = { spawn: Bool; x: Int; }

// The spawn-first literal: field `spawn` must parse like any other name.
pub fn ship_ready(on: Bool) -> Int {
  let s = Ship{ spawn: on; x: 7 };
  if s.spawn { return s.x; };
  return 0;
}

// Negative twin: the same literal shape with a plain field name (the
// engine's rename workaround) stays green either way.
type Crate = { spawn_pos: Bool; y: Int; }

pub fn crate_ready(on: Bool) -> Int {
  let c = Crate{ spawn_pos: on; y: 5 };
  if c.spawn_pos { return c.y; };
  return 0;
}

// Shorthand form with `spawn` FIRST (`Ship{ spawn, x: n }`) must also be
// recognized as a struct literal.
pub fn ship_shorthand(on: Bool) -> Int {
  let spawn = on;
  let x = 3;
  let s = Ship{ spawn, x: x };
  if s.spawn { return s.x; };
  return 0;
}
