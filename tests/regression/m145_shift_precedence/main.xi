// m145 (benchmark relay R-1): C-family/Rust bitwise precedence.
// `a << b | c` used to parse as `a << (b | c)` -- silent wrong values in
// bit-assembly code ((1 << 8) | 2 evaluated to 1024). Shifts now bind
// tighter than `&` > `^` > `|` > comparisons (Rust/C order), while `+`/`*`
// stay tighter than shifts and bitwise stays tighter than comparisons
// (the stdlib's `(n >> hi) & 1 == 1` shape is unchanged).
module m145_shift_precedence;

fn assemble(hi: Int, lo: Int) -> Int {
  // Bit-packing MUST group the shift before the OR.
  return hi << 8 | lo;
}

fn flags_with(n: Int) -> Int {
  // OR of a shift: `3 | 4 << 1` == 3 | 8 == 11.
  return 3 | n << 1;
}

fn compare_shape(n: Int, hi: Int) -> Bool {
  // Bitwise binds tighter than `==` (stdlib popcount shape).
  return (n >> hi) & 1 == 1;
}

fn main() -> Int {
  if 1 << 8 | 2 != 258 { return 1; }
  if 3 | 4 << 1 != 11 { return 2; }
  if 1 << 2 + 1 != 8 { return 3; }
  if 8 & 3 << 1 != 0 { return 4; }

  if assemble(1, 2) != 258 { return 5; }
  if flags_with(4) != 11 { return 6; }

  // 0b1101 >> 1 & 1 == 0; (0b1101 >> 0) & 1 == 1.
  if compare_shape(13, 1) != false { return 7; }
  if compare_shape(13, 0) != true { return 8; }

  // `&`/`^`/`|` are looser than `*` but tighter than comparisons:
  // (2*3) & 4 == 4; 2 & (3*4) == 0; (1^2) | (3&4) == 3.
  if 2 * 3 & 4 != 4 { return 9; }
  if 2 & 3 * 4 != 0 { return 10; }
  if 1 ^ 2 | 3 & 4 != 3 { return 11; }

  return 0;
}
