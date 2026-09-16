// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Cast chain Int8->Int16->Int32->Int64 with elif
fn main() -> Int {
  var x: Int8 = -1;
  var y: Int16 = x as Int16;
  var z: Int32 = y as Int32;
  var w: Int64 = z as Int64;
  var neg_one: Int64 = -1;
  if w == neg_one {
    var neg_one32: Int32 = -1;
    if z == neg_one32 {
      var neg_one16: Int16 = -1;
      if y == neg_one16 {
        var neg_one8: Int8 = -1;
        if x == neg_one8 {
          return 0;
        }
        return 1;
      } elif y + 1 == 0 as Int16 {
        return 2;
      } else {
        return 3;
      }
    } elif z + 1 == 0 as Int32 {
      return 4;
    } else {
      return 5;
    }
  } elif w + 1 == 0 as Int64 {
    return 6;
  } else {
    return 7;
  }
  return 8;
}
