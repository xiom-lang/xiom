// m138 (packages relay): widened UNSIGNED constants must zero-extend.
// `239u8 as Int` sign-extended to -17 because a suffixed literal parses as
// `As(Int(239), UInt8)` and the widening arm had no source-type arm for that
// shape (it defaulted to sext) -- the packages' syslog BOM corruption, where
// `(c as Int) & 0xFF` was the workaround. Runtime UInt8 locals were already
// correct (they resolve through xiom_type_of_local).
module m138_u8_const_widen;

pub type Holder = { b: UInt8; }

fn main() -> Int {
  let a = 239u8 as Int;
  if a != 239 { return 1; }
  let b = 200u8 as Int;
  if b != 200 { return 2; }
  let c = 128u8 as Int;
  if c != 128 { return 3; }
  let paren = (239u8) as Int;
  if paren != 239 { return 4; }

  let x: UInt8 = 239;
  if x as Int != 239 { return 5; }

  // Field source (ntriples shape): a UInt8 STRUCT FIELD must zero-extend.
  var h = Holder{ b: 239u8 };
  if h.b as Int != 239 { return 9; }

  // Signed narrow sources keep sign-extending.
  let d = -17i8 as Int;
  if d != -17 { return 6; }
  let e = 239 as UInt8 as Int;
  if e != 239 { return 7; }

  // The mask workaround still works (and is no longer required).
  let masked = (a) & 0xFF;
  if masked != 239 { return 8; }

  return 0;
}
