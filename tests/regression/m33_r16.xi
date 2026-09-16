// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

enum InnerResult { InOk(val: Int), InErr(msg: Str) }
enum OuterResult { OutOk(content: InnerResult), OutErr(msg: Str) }
fn main() -> Int {
  var a = OuterResult.OutOk(InnerResult.InOk(42));
  match a {
    OutOk(inner) => { match inner { InOk(n) => { if n != 42 { return 1; } } InErr(_) => { return 2; } } }
    OutErr(_) => { return 3; }
  }
  var b = OuterResult.OutOk(InnerResult.InErr("inner"));
  match b {
    OutOk(inner) => { match inner { InOk(_) => { return 4; } InErr(_) => {} } }
    OutErr(_) => { return 5; }
  }
  var c = OuterResult.OutErr("outer");
  match c {
    OutOk(_) => { return 6; }
    OutErr(_) => {}
  }
  return 0;
}
