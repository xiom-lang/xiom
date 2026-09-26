// m146 (playground relay): a let-bound extern returning Int32 must keep its
// SIGNEDNESS. `strcmp("a", "b")` returns a negative int; the binding path
// recorded the call-return XIOM type without updating `signed_locals`, so
// the narrow load widened `zext i32 -1 to i64` == 4294967295 and
// `r < 0` was false. Same class as net.tcp_connect reporting Ok on a
// refused connection (its C `int` result was also widened unsigned; that
// pin-level wrapper additionally declares `-> Int`, see COMPILER_BUGS).
module m146_signed_extern_result;

extern "C" {
  fn strcmp(a: *UInt8, b: *UInt8) -> Int32;
}

fn main() -> Int
  requires: true
{
  var a: [2]UInt8;
  a[0] = 97;  // 'a'
  a[1] = 0;
  var b: [2]UInt8;
  b[0] = 98;  // 'b'
  b[1] = 0;

  unsafe {
    // strcmp("a","b") < 0 -- the negative i32 must sign-extend.
    let r = strcmp(&a as *UInt8, &b as *UInt8);
    if r < 0 { } else { return 3; }

    // Equal strings: 0 stays 0.
    let eq = strcmp(&a as *UInt8, &a as *UInt8);
    if eq == 0 { } else { return 4; }

    // Reverse comparison is positive.
    let rev = strcmp(&b as *UInt8, &a as *UInt8);
    if rev > 0 { } else { return 5; }
  }
  return 0;
}
