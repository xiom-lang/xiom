// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// BUG 50 regression: generic container names with POINTER type args must
// not leak `*` into LLVM identifiers (%struct.Result__*UInt8__AllocError ->
// clang "expected '=' after name"). The mono names the container from the
// struct-payload rule; the pointer arg now sanitizes to a valid id.
module m37_bug50_ptr_container_name
use xiom.core;

type AllocError = {
  message: Str;
}

fn allocate(l: Int) -> Result<*mut UInt8, AllocError> {
  if l <= 0 {
    return Err(AllocError{ message: "bad"; });
  }
  unsafe {
    var p: *mut UInt8 = malloc(l);
    return Ok(p);
  }
}

fn main() -> Int {
  let r = allocate(16);
  match r {
    Ok(p) => {
      if p == null {
        return 1;
      }
      return 0;
    }
    Err(e) => {
      let _ = e;
      return 2;
    }
  }
}
