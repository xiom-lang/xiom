// m266 (REL-20261010-2124-xvector + REL-20261010-1802 gate): an unresolved
// `use` must produce EXACTLY ONE clear error at the import line and never
// continue silently. The module below is deliberately absent from every
// stdlib/package tree; `xiom.core.error` now resolves, so the fixture uses a
// name that cannot start resolving later. The W004 cascade lock lives in
// `m259_w004_suppressed_on_error_tainted_body`.
// Negative lock: checker_locks::m266_unresolved_import_one_clear_error.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m266_unresolved_import

use xiom.core.error_does_not_exist;

pub fn main() -> Int {
  return 0;
}
