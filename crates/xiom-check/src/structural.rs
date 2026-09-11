// XIOM -- Checker Structural Type Identity (Stage 2c)
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

//! Compatibility shim: the canonical type-name parser/renderer now lives in
//! [`xiom_ast::structural`] so the checker, codegen, driver and LSP all share
//! ONE implementation (Stage 2c). Re-exported here to keep the checker's
//! historical `crate::structural::*` paths stable.

pub use xiom_ast::structural::*;
