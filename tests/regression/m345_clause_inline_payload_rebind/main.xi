// m345 (REL-20261011-1411-engine): the contract clause payload rebind
// (`result is Ok` / `result is Some`) hard-coded an i64-HANDLE load of the
// payload field. The base Result/Option layout carries i64 handles, but the
// concrete monomorphs (`ensure_concrete_result`/`ensure_concrete_option`,
// crates/xiom-codegen/src/lib.rs) carry the payload INLINE
// (%struct.Result__Vec__IOError = { i64, %struct.Vec, %struct.IOError }).
// The i64 load of field 1 therefore grabbed the Vec's DATA POINTER; the
// `.len()` consequence inttoptr'd it to %struct.Vec* and read offset+8 --
// the first file bytes -- as the length, so a payload whose bytes 8..15
// carry bit 63 set produced a NEGATIVE length and tripped the correct
// tautology `result.len() >= 0` (the engine's grid.ppm abort, io.xi:947).
// Arms A (file-read `result.len()`), B (the `.value`-reader consequence,
// field-read 3338-3350) and C (concrete Option payload field read) are RED
// pre-fix (contract violation / wrong evaluation); post-fix the clause GEPs
// the INLINE struct (no inttoptr-as-handle), every arm green, rc 0.
// D pins the BASE `Option[Vec[UInt8]]` handle layout as unchanged (green in
// both states); E/F are positive-bytes and Err-vacuous controls.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module m345_clause_inline_payload_rebind;

use xiom.io;
use xiom.convert;

type IOError = {
  message: Str;
  code: Int;
}

type Blob = {
  data: Vec[UInt8];
}

extern "C" {
  fn xiom_read_file(path: *UInt8) -> *UInt8;
  fn xiom_file_size(path: *UInt8) -> Int;
  fn xiom_free(ptr: *UInt8);
}

// Arm A: the engine report's shape -- a Result[Vec[UInt8], <struct error>]
// file reader carrying the canonical clause the stdlib workaround dd352f6
// replaced (io.xi:947). Restoring it here keeps the compiler-side root
// cause pinned independently of the stdlib text.
pub fn own_read_file_bytes(path: Str) -> Result[Vec[UInt8], IOError]
  ensures: result is Ok => result.len() >= 0
{
  let c_path = path.c_str();
  let ptr: *UInt8;
  let size: Int;
  unsafe {
    ptr = xiom_read_file(c_path);
    if ptr == 0 {
      return Err(IOError{ message: "failed to read file: " + path, code: 1 });
    }
    size = xiom_file_size(c_path);
  }
  var buf: Vec[UInt8] = Vec[UInt8].with_capacity(size as UInt);
  unsafe {
    var i = 0;
    while i < size {
      buf.push(*(ptr.offset(i)));
      i = i + 1;
    }
    xiom_free(ptr);
  }
  Ok(buf)
}

// Arm B: the same reader with the `.value`-reader consequence -- inside the
// `is Ok` consequence the rebind makes `result.value` the payload itself,
// and the pre-fix unbox path inttoptr'd the raw field-1 bits as a boxed Vec
// handle (payload bytes 8..15 became the length).
pub fn own_read_file_bytes_value(path: Str) -> Result[Vec[UInt8], IOError]
  ensures: result is Ok => result.value.len() >= 0
{
  let c_path = path.c_str();
  let ptr: *UInt8;
  let size: Int;
  unsafe {
    ptr = xiom_read_file(c_path);
    if ptr == 0 {
      return Err(IOError{ message: "failed to read file: " + path, code: 1 });
    }
    size = xiom_file_size(c_path);
  }
  var buf: Vec[UInt8] = Vec[UInt8].with_capacity(size as UInt);
  unsafe {
    var i = 0;
    while i < size {
      buf.push(*(ptr.offset(i)));
      i = i + 1;
    }
    xiom_free(ptr);
  }
  Ok(buf)
}

// Arm C: the concrete Option twin -- the struct payload is INLINE
// (%struct.Option__Blob = { i64, %struct.Blob }); pre-fix `result.data`
// read a literal 0 (wrong evaluation, rc 1 at runtime), post-fix the field
// read lowers against the bound inline struct and evaluates the real Vec.
pub fn pick_blob(v: Vec[UInt8]) -> Option[Blob]
  ensures: result is Some => result.data.len() == 24
{
  return Some(Blob{ data: v });
}

// Arm D control: `Option[Vec[UInt8]]` stays on the BASE %struct.Option
// handle layout (Vec is not a registered struct payload), so the historical
// boxed-handle rebind is correct -- green in both states.
pub fn nonempty_or_none(v: Vec[UInt8]) -> Option[Vec[UInt8]]
  ensures: result is Some => result.len() >= 0
{
  if v.len() == 0 {
    return None;
  }
  return Some(v);
}

// Arm E control: a Result wrap over an all-positive payload -- green in
// both states (pre-fix the misread length happens to be positive).
pub fn wrap(v: Vec[UInt8]) -> Result[Vec[UInt8], IOError]
  ensures: result is Ok => result.len() >= 0
{
  return Ok(v);
}

fn payload_with_high_bit() -> Vec[UInt8] {
  var out = Vec[UInt8].with_capacity(24 as UInt);
  var i: UInt = 0;
  while i < 24 {
    if i == 15 {
      // Byte 15 = 0x80 -> bit 63 of the little-endian u64 at bytes 8..15.
      out.push(128 as UInt8);
    } else if i < 8 {
      out.push((65 + i) as UInt8);
    } else {
      out.push(0 as UInt8);
    }
    i = i + 1;
  }
  return out;
}

fn payload_positive() -> Vec[UInt8] {
  var out = Vec[UInt8].with_capacity(24 as UInt);
  var i: UInt = 0;
  while i < 24 {
    out.push((65 + i) as UInt8);
    i = i + 1;
  }
  return out;
}

pub fn main() -> Int {
  var fails: Int = 0;
  let path = "e2e_m345_payload.bin";

  let payload = payload_with_high_bit();
  let wr = io.write_file_bytes(path, payload);
  if wr.is_err {
    io.println("m345: write failed");
    return 9;
  }

  let ra = own_read_file_bytes(path);
  if ra.is_err {
    fails = fails + 1;
    io.println("m345 [A]: read returned Err");
  } else {
    if ra.value.len() != 24 {
      fails = fails + 1;
      io.println("m345 [A]: len " + convert.int_to_string(ra.value.len()));
    }
    if ra.value[0] != (65 as UInt8) {
      fails = fails + 1;
      io.println("m345 [A]: byte 0 wrong");
    }
    if ra.value[15] != (128 as UInt8) {
      fails = fails + 1;
      io.println("m345 [A]: byte 15 wrong");
    }
  }

  let rb = own_read_file_bytes_value(path);
  if rb.is_err {
    fails = fails + 1;
    io.println("m345 [B]: read returned Err");
  } else {
    if rb.value.len() != 24 {
      fails = fails + 1;
      io.println("m345 [B]: len " + convert.int_to_string(rb.value.len()));
    }
  }

  let rc = pick_blob(payload);
  if rc.is_some {
    if rc.value.data.len() != 24 {
      fails = fails + 1;
      io.println("m345 [C]: data len " + convert.int_to_string(rc.value.data.len()));
    }
  } else {
    fails = fails + 1;
    io.println("m345 [C]: returned None");
  }

  let rd = nonempty_or_none(payload);
  if rd.is_some {
    if rd.value.len() != 24 {
      fails = fails + 1;
      io.println("m345 [D]: len " + convert.int_to_string(rd.value.len()));
    }
  } else {
    fails = fails + 1;
    io.println("m345 [D]: returned None");
  }

  let re = wrap(payload_positive());
  if re.is_err {
    fails = fails + 1;
    io.println("m345 [E]: wrap returned Err");
  } else {
    if re.value.len() != 24 {
      fails = fails + 1;
      io.println("m345 [E]: len " + convert.int_to_string(re.value.len()));
    }
  }

  // Arm F: the Err path stays vacuous (missing file).
  let rf = own_read_file_bytes("e2e_m345_missing_file.bin");
  if !rf.is_err {
    fails = fails + 1;
    io.println("m345 [F]: missing file returned Ok");
  }

  io.remove_file(path);

  io.println("m345_clause_inline_payload_rebind: " + convert.int_to_string(fails) + " failing checks");
  return fails;
}
