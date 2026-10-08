// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m226 (C-ORBIT-02): index-assign into a Vec of CONTAINER elements
// (`Vec[Option[Int]]`, `Vec[Result[Int, Int]]`) boxed the value and stored the
// 8-byte handle through the scalar elem_store path; the 16-byte slot then held
// a pointer where the tag was expected and the match ran NO arm (ORBITDB
// WAL/transaction probe V1/V3/V6/V8 silently skipped). Container elements must
// memcpy the erased generic struct ("Option[Int]" -> %struct.Option).

module m226_vec_option_assign

fn main() -> Int {
  var v = Vec[Option[Int]].new();
  v.push(None);
  v[0] = Some(44);
  match v[0] {
    None => { return 1; }
    Some(x) => { if x != 44 { return 2; } }
  }

  var r = Vec[Result[Int, Int]].new();
  r.push(Err(1));
  r[0] = Ok(7);
  match r[0] {
    Err(e) => { return 3 + e; }
    Ok(x) => { if x != 7 { return 4; } }
  }

  return 0;
}
