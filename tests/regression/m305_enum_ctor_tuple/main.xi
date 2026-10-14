// m305 (REL-20261011-1219-orbitdb; C-ORBIT-08 PART A): a tuple CONTAINING AN
// ENUM built from an enum CONSTRUCTOR call (`Ok((Color.Custom(n), n + 3))`)
// miscompiled. Both tuple-element namers fell back to the i64-erased element
// type, so the arm built a 16-byte Tuple__Int__Int: the enum TAG landed in
// slot 0 and the payload was DROPPED, while the fn signature / read side used
// the 24-byte Tuple__Color__Int -- the Ok payload read a tag as the Int
// component and the enum payload read past the allocation (silent wrong
// values; pre-fix v0.64.3 crashed STATUS_BREAKPOINT). The fix names enum
// CTOR-call elements by the parent enum key, matching the variant-REF path
// and the signature side. Controls: the constant-variant arms, the Err arm,
// a plain let-bound (Color, Int) tuple, and the enum-alone workaround shape.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module m305_enum_ctor_tuple;

use xiom.io;
use xiom.convert;

pub type Color = enum { Red, Blue, Custom(n: Int) }

fn pick(c: Color) -> Result[(Color, Int), Int] {
  match c {
    Red => { return Ok((Color.Red, 1)); }
    Blue => { return Ok((Color.Blue, 2)); }
    Custom(n) => { return Ok((Color.Custom(n), n + 3)); }
  }
}

fn pick_err(k: Int) -> Result[(Color, Int), Int] {
  return Err(k);
}

fn main() -> Int {
  var fails: Int = 0;

  // A: the ctor-call arm -- the Int component must read n+3 and the enum
  // payload must read n (pre-fix: n read the enum TAG 2 / garbage and the
  // enum payload read past the 16-byte box and printed 8).
  let ra = pick(Color.Custom(5));
  match ra {
    Err(_) => {
      fails = fails + 1;
      io.println("  [A] ctor arm returned Err");
    }
    Ok(pair) => {
      let n = pair.1;
      if n != 8 {
        fails = fails + 1;
        io.println("  [A] pair.1 wrong: " + convert.int_to_string(n) + " (expected 8)");
      }
      match pair.0 {
        Red => {
          fails = fails + 1;
          io.println("  [A] payload arm = Red");
        }
        Blue => {
          fails = fails + 1;
          io.println("  [A] payload arm = Blue");
        }
        Custom(m) => {
          if m != 5 {
            fails = fails + 1;
            io.println("  [A] payload wrong: " + convert.int_to_string(m) + " (expected 5)");
          }
        }
      }
    }
  }

  // B: constant-variant arms (green pre-fix; must stay green).
  let rb = pick(Color.Red);
  match rb {
    Err(_) => {
      fails = fails + 1;
      io.println("  [B] Red arm returned Err");
    }
    Ok(pair) => {
      if pair.1 != 1 {
        fails = fails + 1;
        io.println("  [B] Red pair.1 wrong: " + convert.int_to_string(pair.1) + " (expected 1)");
      }
      match pair.0 {
        Red => {}
        Blue => {
          fails = fails + 1;
          io.println("  [B] Red payload arm = Blue");
        }
        Custom(_) => {
          fails = fails + 1;
          io.println("  [B] Red payload arm = Custom");
        }
      }
    }
  }
  let rc2 = pick(Color.Blue);
  match rc2 {
    Err(_) => {
      fails = fails + 1;
      io.println("  [B] Blue arm returned Err");
    }
    Ok(pair) => {
      if pair.1 != 2 {
        fails = fails + 1;
        io.println("  [B] Blue pair.1 wrong: " + convert.int_to_string(pair.1) + " (expected 2)");
      }
      match pair.0 {
        Blue => {}
        Red => {
          fails = fails + 1;
          io.println("  [B] Blue payload arm = Red");
        }
        Custom(_) => {
          fails = fails + 1;
          io.println("  [B] Blue payload arm = Custom");
        }
      }
    }
  }

  // C: Err arm (green pre-fix; must stay green).
  let re = pick_err(9);
  match re {
    Ok(_) => {
      fails = fails + 1;
      io.println("  [C] pick_err returned Ok");
    }
    Err(k) => {
      if k != 9 {
        fails = fails + 1;
        io.println("  [C] Err payload wrong: " + convert.int_to_string(k) + " (expected 9)");
      }
    }
  }

  // D: control -- a plain let-bound (Color, Int) tuple (green pre-fix).
  let plain = (Color.Custom(7), 70);
  if plain.1 != 70 {
    fails = fails + 1;
    io.println("  [D] plain.1 wrong: " + convert.int_to_string(plain.1) + " (expected 70)");
  }
  match plain.0 {
    Custom(m) => {
      if m != 7 {
        fails = fails + 1;
        io.println("  [D] plain.0 payload wrong: " + convert.int_to_string(m) + " (expected 7)");
      }
    }
    Red => {
      fails = fails + 1;
      io.println("  [D] plain.0 arm = Red");
    }
    Blue => {
      fails = fails + 1;
      io.println("  [D] plain.0 arm = Blue");
    }
  }

  // E: control -- the accepted workaround shape (enum alone, green pre-fix).
  let solo = Color.Custom(5);
  match solo {
    Custom(m) => {
      if m != 5 {
        fails = fails + 1;
        io.println("  [E] solo payload wrong: " + convert.int_to_string(m) + " (expected 5)");
      }
    }
    Red => {
      fails = fails + 1;
      io.println("  [E] solo arm = Red");
    }
    Blue => {
      fails = fails + 1;
      io.println("  [E] solo arm = Blue");
    }
  }

  io.println("m305_enum_ctor_tuple: " + convert.int_to_string(fails) + " failing checks");
  return fails;
}
