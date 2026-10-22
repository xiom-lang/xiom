// m1827 item B (REL-20261022-1645-packages): cross-module
// `Result[(Handle, Vec[Int]), Str]`. The tuple literal used to fall back to
// the enclosing `%struct.Result` layout, coercing the Vec element to its data
// pointer and dropping len/cap/elem_size at the module boundary.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m1827_vec_in_tuple_lib

pub type Handle = Int;

pub fn make_handle() -> Result[Handle, Str] {
  return Ok(7);
}

pub fn pair() -> Result[(Handle, Vec[Int]), Str] {
  let h = make_handle()?;
  var v = Vec[Int].new();
  v.push(11);
  v.push(22);
  return Ok((h, v));
}
