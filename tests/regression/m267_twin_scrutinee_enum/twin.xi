// m267 fixture: the foreign same-named type (mirrors the XVECTOR inline
// CoreError stand-in that bound the name in xiom.db.error).
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module twin

pub enum CoreError {
  NotFound,
  Corruption(msg: Str),
  InvalidInput(msg: Str),
}
