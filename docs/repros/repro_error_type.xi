// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// REPRO R1 (BUG 27 #6): "Error" as a pub type name in a catalog module.
// Wave-5 error/chain.xi originally used "pub type Error" and emitted
// "unknown type 'Error' defaulting to i64" + corrupted cross-module codegen.
module repro_error_type

pub type Error = {
  code: Int;
  message: Str;
}

pub fn make_error(code: Int) -> Error {
  return Error{ code: code; message: "err"; };
}

pub fn error_code(e: &Error) -> Int {
  return e.code;
}