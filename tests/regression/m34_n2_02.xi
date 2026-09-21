// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-N2-02: 10-deep while-with-while -- pushes loop nesting limit
fn main() -> Int {
  var i0 = 0;
  var sum = 0;
  while i0 < 2 {
    var i1 = 0;
    while i1 < 2 {
      var i2 = 0;
      while i2 < 2 {
        var i3 = 0;
        while i3 < 2 {
          var i4 = 0;
          while i4 < 2 {
            var i5 = 0;
            while i5 < 2 {
              var i6 = 0;
              while i6 < 2 {
                var i7 = 0;
                while i7 < 2 {
                  var i8 = 0;
                  while i8 < 2 {
                    var i9 = 0;
                    while i9 < 2 {
                      sum = sum + 1;
                      i9 = i9 + 1;
                    }
                    i8 = i8 + 1;
                  }
                  i7 = i7 + 1;
                }
                i6 = i6 + 1;
              }
              i5 = i5 + 1;
            }
            i4 = i4 + 1;
          }
          i3 = i3 + 1;
        }
        i2 = i2 + 1;
      }
      i1 = i1 + 1;
    }
    i0 = i0 + 1;
  }
  if sum == 1024 { return 0; }
  return 1;
}
