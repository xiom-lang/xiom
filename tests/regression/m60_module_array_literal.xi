// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M60 (R2, M58 residual): module-level mutable arrays with all-constant
// LITERAL initializers. Old path routed them to the runtime-init ctor,
// whose compile_expr on the array literal yields an i8* heap buffer --
// storing a 'ptr' into an [N x T] global was invalid IR ("%tmp defined
// with type 'ptr' but expected '[4 x i64]'"). Fix: expr_is_const_init
// accepts all-constant array literals and global_const_init renders the
// LLVM constant aggregate (decl.rs / lib.rs; types.rs copy kept in sync).
module m60_module_array_literal

var _a: [4]Int = [10, 20, 30, 40];
var _neg: [3]Int = [-1, -2, -3];
var _big: [256]Int = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127, 128, 129, 130, 131, 132, 133, 134, 135, 136, 137, 138, 139, 140, 141, 142, 143, 144, 145, 146, 147, 148, 149, 150, 151, 152, 153, 154, 155, 156, 157, 158, 159, 160, 161, 162, 163, 164, 165, 166, 167, 168, 169, 170, 171, 172, 173, 174, 175, 176, 177, 178, 179, 180, 181, 182, 183, 184, 185, 186, 187, 188, 189, 190, 191, 192, 193, 194, 195, 196, 197, 198, 199, 200, 201, 202, 203, 204, 205, 206, 207, 208, 209, 210, 211, 212, 213, 214, 215, 216, 217, 218, 219, 220, 221, 222, 223, 224, 225, 226, 227, 228, 229, 230, 231, 232, 233, 234, 235, 236, 237, 238, 239, 240, 241, 242, 243, 244, 245, 246, 247, 248, 249, 250, 251, 252, 253, 254, 255];

fn main() -> Int {
  var s = 0;
  var i = 0;
  while i < 4 {
    s = s + _a[i];
    i = i + 1;
  }
  if s != 100 { return 1; }
  var ns = 0;
  i = 0;
  while i < 3 {
    ns = ns + _neg[i];
    i = i + 1;
  }
  if ns != -6 { return 3; }
  var bs = 0;
  i = 0;
  while i < 256 {
    bs = bs + _big[i];
    i = i + 1;
  }
  // sum(0..255) = 32640
  if bs != 32640 { return 4; }
  if _big[255] != 255 { return 5; }
  // index writes into the literal-backed global still work (M58 path)
  _big[10] = 1000;
  if _big[10] != 1000 { return 6; }
  return 0;
}
