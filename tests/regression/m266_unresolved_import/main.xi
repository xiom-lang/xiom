// m266 (REL-20261010-2124-xvector + REL-20261010-1802 gate): an unresolved
// `use` must produce EXACTLY ONE clear error at the import line and never
// continue silently. This fixture is the orbitdb phantom shape: the import
// does not resolve anywhere, and the undeclared `CoreError` used in `fold`'s
// signature must NOT degrade into phantom W004 unreachable-arm warnings (nor
// a cross-module T001 when a twin exists).
// Negative lock: checker_locks::m266_unresolved_import_one_clear_error.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m266_unresolved_import

use xiom.core.error;

pub enum DbError {
  NotFound,
  Corruption,
}

pub fn fold(e: &CoreError) -> DbError {
  match e {
    NotFound => DbError.NotFound,
    Corruption(_) => DbError.Corruption,
    _ => DbError.NotFound,
  }
}
