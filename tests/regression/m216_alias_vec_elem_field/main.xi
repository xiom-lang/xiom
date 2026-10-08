// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m216 (C-PULSE-11): a type alias to a package-style struct
// (`pub type Store = pkg.SessionStore;`) used as a Vec element must resolve
// in the element-struct lookup. Pre-fix `v[0].field` read the constant 0
// (resolve_vec_elem_type did not follow type_aliases), `Vec[Store].new()`
// sized its buffer by the unknown-name default (8), and the field read off
// a cross-module alias element silently miscompiled.

module pkg {
  pub type SessionStore = {
    ttl: Int;
    count: Int;
  }

  pub fn session_store_new(ttl: Int) -> SessionStore {
    return SessionStore { ttl: ttl, count: 7 };
  }
}

module app {
  pub type Store = pkg.SessionStore;

  fn pick(s: Store) -> Int {
    return s.ttl;
  }

  pub fn run() -> Int {
    var v: Vec[Store] = Vec[Store].new();
    v.push(pkg.session_store_new(60000));
    if v[0].ttl != 60000 { return 1; }
    if v[0].count != 7 { return 2; }
    if pick(v[0]) != 60000 { return 3; }
    return 0;
  }
}

fn main() -> Int {
  return app.run();
}
