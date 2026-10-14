// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m304 lock (REL-20261010-1550-orbitdb-2): a standalone check of a
// self-referential module failed T001 "expected Engine, found module" --
// the declared chain's segments were flattened into the module map
// (`module m304_module_local_shadow.engine` registers "engine" as a
// submodule export of `m304_module_local_shadow`) and
// check_module_field_access tried module-qualified resolution BEFORE any
// local-scope lookup, so `m304_module_local_shadow.engine` on a PARAMETER
// named exactly like the chain's first segment typed as the submodule
// ("module"). Locals now shadow module aliases in expression position,
// mirroring the Ident arm's locals-first precedence. The parameter/local
// pair mirrors the orbitdb witness: `engine_insert(&mut db.engine, ...)`
// with `module xiom.db.engine`.
module m304_module_local_shadow.engine

type Engine = {
  root: Int;
}

type Database = {
  engine: Engine;
}

fn engine_new(order: Int) -> Engine {
  return Engine{ root: order };
}

fn engine_root(eng: &mut Engine) -> Int {
  return eng.root;
}

// Witness shape: argument 1 (`&mut <local>.engine`) must type as Engine --
// pre-fix it typed as "module" (the flattened chain's submodule export).
fn db_root(m304_module_local_shadow: &mut Database) -> Int {
  return engine_root(&mut m304_module_local_shadow.engine);
}

fn db_open(order: Int) -> Database {
  return Database{ engine: engine_new(order) };
}

fn main() -> Int {
  var db = db_open(5);
  if db_root(&mut db) != 5 { return 1; }
  return 0;
}
