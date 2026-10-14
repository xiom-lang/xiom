// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m310 lock (REL-20261010-1549-packages-7): the installed-dep side of the
// range-spec fix. The app's xiom.toml asks for ">=0.2.0"; the old resolver
// matched installed dirs with a literal starts_with and only exact specs
// resolved, leaving the import unresolved.
module xiom.rate

pub fn rate_new() -> Int {
  return 1;
}
