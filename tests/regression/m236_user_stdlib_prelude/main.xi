// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m236 (m230): the entry program imports only a user module; the stdlib
// reachability is transitive (h_user -> xiom.encoding). The prelude must
// still load so the imported module's catalog bodies check cleanly.
module m236_user_stdlib_prelude

use m236_h_user;

fn main() -> Int {
  return m236_h_user.p();
}
