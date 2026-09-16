// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// gzip-DECOMPRESS regression (2026-08-19, queue item 1): the catalog
// compress.gzip_decompress crashed with 0xC0000005 (crc32 element-load AV
// from `let decompressed = decoded.value;` -- the payload-FIELD access on a
// Result local bound the raw BOXED pointer as i64; `&decompressed` passed
// the i64 slot address as %struct.Vec* -> stack garbage len/elem_size ->
// 8-byte element load). The payload field is now unboxed via
// field_payload_xiom (inttoptr + load %struct.Vec).
//
// Second root: `let compressed = if level == 0 { _store_encode(data) }
// else { rle_encode(data) };` -- the if-expression result slot was
// hardcoded i64, so the Vec VALUE degraded to field-0-as-i64 (data ptr);
// `compressed.len()` became xiom_str_len and `compressed[i]` compiled to
// literal 0 -> gzip payload of all zeros -> wrong decode (len 3 of 0x00).
// The if-expression result type is now inferred from the arm tails
// (struct > pointer > i64), and the binding records the arm-tail XIOM
// type so Vec.len/index dispatch correctly.
module m37_gzip_roundtrip
use xiom.compress;
use xiom.io;

fn main() -> Int {
  var data = Vec[UInt8].new();
  data.push(72u8);
  data.push(101u8);
  data.push(108u8);
  data.push(108u8);
  data.push(111u8);
  var compressed = compress.gzip_compress(&data);
  match compressed {
    Ok(c) => {
      if c.len() < 18 { return 1; }
      var decompressed = compress.gzip_decompress(&c);
      match decompressed {
        Ok(result) => {
          if result.len() != data.len() { return 2; }
          var i = 0;
          while i < result.len() {
            if result[i] != data[i] { return 3; }
            i = i + 1;
          }
          return 0;
        }
        Err(e) => { io.println("gzip: " + e); return 4; }
      }
    }
    Err(e) => { io.println("gzip: " + e); return 5; }
  }
}
