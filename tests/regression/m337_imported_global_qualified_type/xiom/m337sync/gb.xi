// m337 (XVC-C-17): same-leaf collision owner `gb` -- Widget{beta;gamma}
// (two fields) collides with ga.Widget{alpha}. A qualified global typed
// `gb.Widget` must use the 16-byte layout; binding ga's 8-byte layout
// drops gamma and miscounts beta.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module xiom.m337sync.gb;

pub type Widget = { beta: Int; gamma: Int; }
