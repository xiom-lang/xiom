// m131 (C1, generic-mono ABI): the call/return ABI follows the
// MONOMORPHISED instantiation, not the erased generic signature. Covers the
// stdlib and packages lanes' cross-type callback matrix: generic U=Str /
// U=Float64 through fn-typed params, `Vec[U]` generic returns, and
// `array.map` by value. Same-type callbacks stay green (fp2_multi shape).
module m131_generic_fn_callback_abi;

use xiom.array;
use xiom.convert;

// array.map[T, U, const N] takes `f: fn(T) -> U` BY VALUE.
fn to_s_val(x: Int) -> Str { return int_to_string(x); }
fn to_fv(x: Int) -> Float64 { return x as Float64; }

// conv/map_to take `f: fn(&T) -> U` BY REFERENCE.
fn to_s(x: &Int) -> Str { return int_to_string(*x); }

pub fn conv[T, U](x: T, f: fn(&T) -> U) -> U {
  return f(&x);
}

pub fn map_to[T, U](v: &Vec[T], f: fn(&T) -> U) -> Vec[U] {
  let out = Vec[U].new();
  let i = 0;
  while (i < v.len()) {
    let x: T = v[i];
    out.push(f(&x));
    i = i + 1;
  }
  return out;
}

fn main() -> Int {
  // by-value cross-type callbacks
  let a = [3, 1, 2];
  let b = array.map(a, to_s_val);
  if b[0] != "3" { return 1; }
  let c = array.map(a, to_fv);
  if c[2] != 2.0 { return 2; }

  // generic U = Str, by-ref callback
  let d = conv(7, to_s);
  if d != "7" { return 3; }

  // generic Vec[U] return with U = Str
  let v = Vec[Int].new();
  v.push(7);
  v.push(8);
  let w = map_to(v, to_s);
  if w.len() != 2 { return 4; }
  let a0: Str = w[0];
  if a0 != "7" { return 5; }

  return 0;
}
