// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// REL-20261104: v0.65.6 freed a returned Str buffer at scope exit when the
// value was moved out through a match/if block arm or an enum-variant
// construction. stdlib `http_build_request` then returned Ok(r) with stale
// bytes, and orbitdb's tuple2 text round-trip failed fuzz seed 1000 (rc 19).
//
// The fixture rebuilds the exact shapes:
//   * `request = request + ...` accumulator returned as `Ok(request)` from a
//     match arm;
//   * the same accumulator moved into a returned `MaybeText.Text(text)`;
// and churns the allocator before reading so a freed buffer is overwritten.
module rel20261104_str_rebind_chain

enum MaybeText {
  Empty,
  Text(Str),
}

fn build_result(n: Int) -> Result[Str, Str] {
  var request = "GET /";
  var i = 0;
  while i < n {
    request = request + "\r\n";
    request = request + "Host: example.com";
    i = i + 1;
  }
  match n {
    0 => { Ok("zero") }
    _ => { Ok(request) }
  }
}

fn build_variant(n: Int) -> MaybeText {
  var text = "ab";
  var i = 0;
  while i < n {
    text = text + "ab";
    i = i + 1;
  }
  return MaybeText.Text(text);
}

fn churn(n: Int) -> Int {
  var s = "";
  var i = 0;
  while i < n {
    s = s + "0123456789abcdef";
    i = i + 1;
  }
  return s.len();
}

fn main() -> Int {
  var i = 0;
  while i < 64 {
    match build_result(3) {
      Ok(r) => {
        let dirty = churn(8);
        if dirty != 128 { return 1; }
        if r.len() != 62 { return 2; }
        if r.starts_with("GET /") != true { return 3; }
        if r.ends_with("example.com") != true { return 4; }
      }
      Err(_) => { return 5; }
    }
    match build_variant(4) {
      Text(t) => {
        let dirty2 = churn(8);
        if dirty2 != 128 { return 6; }
        if t.len() != 10 { return 7; }
        if t != "ababababab" { return 8; }
      }
      Empty => { return 9; }
    }
    i = i + 1;
  }
  return 0;
}
