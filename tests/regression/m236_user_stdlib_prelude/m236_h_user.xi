// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m236 (m230): a USER module importing a stdlib module must trigger the stdlib
// catalog PRELUDE (core/string/collections). Before the fix the prelude gate
// looked only at the entry program's own `use` list, so this chain ran the
// on-demand catalog-body check of `xiom.encoding` without the core container
// declarations and hard-failed: 3x T001 "cannot call 'get' on this expression"
// at encoding.xi 409/419/466, compilation failed.
module m236_h_user

use xiom.encoding;

pub fn p() -> Int {
  return 0;
}
