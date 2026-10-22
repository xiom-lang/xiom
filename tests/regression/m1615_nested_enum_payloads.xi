// m1615 (REL-20261022-1615-xvector): nested enum patterns inside
// Option/Result must check the inner discriminant and bind its payload.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Before the fix `Some(Num(n))` / `Ok(Num(n))` selected the first Some/Ok
// arm without checking the inner variant, and `n` was never bound (read 0).
// This locks Float64 and Str payloads through both Option and Result.

module m1615_nested_enum_payloads

use xiom.io;
use xiom.convert;
use xiom.serialize.json;

enum Inner {
  Num(Float64)
  Text(Str)
}

fn main() -> Int {
  let o: Option[Inner] = Some(Num(3.75));
  match o {
    Some(Num(n)) => {
      io.println("opt_n=" + convert.float_to_string(n));
      if n != 3.75 { return 1; }
    }
    _ => { return 2; }
  }

  let r: Result[Inner, Str] = Ok(Num(2.5));
  match r {
    Ok(Num(n)) => {
      io.println("res_n=" + convert.float_to_string(n));
      if n != 2.5 { return 3; }
    }
    _ => { return 4; }
  }

  let s: Option[Inner] = Some(Text("nested-str"));
  match s {
    Some(Text(v)) => {
      io.println("opt_s=" + v);
      if v != "nested-str" { return 5; }
    }
    _ => { return 6; }
  }

  let t: Result[Inner, Str] = Ok(Text("res-str"));
  match t {
    Ok(Text(v)) => {
      io.println("res_s=" + v);
      if v != "res-str" { return 7; }
    }
    _ => { return 8; }
  }

  let doc = json.json_parse("{\"d\":3.73975348472595}");
  match doc {
    Ok(v) => {
      match json.json_get(v, "d") {
        Some(JsonValue.Number(n)) => {
          io.println("json_n=" + convert.float_to_string(n));
          if n != 3.73975348472595 { return 9; }
        }
        _ => { return 10; }
      }
    }
    Err(_) => { return 11; }
  }

  io.println("m1615 ok");
  return 0;
}
