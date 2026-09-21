// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-Z29: pointer+unsafe+cast+generic+Result+match+module+contract+derive+compound_assign+while+enum
type Span = { id: Int; count: Int; } derive[Eq]
enum SpanResult { Ok, Overflow, NullPtr, OutOfBounds(idx: Int) }
fn span_access[T](s: Span, idx: Int) -> Result[Int, Str]
  requires: s.count >= 0
{
  if idx < 0 || idx >= s.count { return Err("bounds"); }
  return Ok(s.id + idx);
}
fn classify_span(s: Span, idx: Int) -> SpanResult {
  if s.count == 0 { return SpanResult.NullPtr; }
  if idx < 0 || idx >= s.count { return SpanResult.OutOfBounds(idx); }
  return SpanResult.Ok;
}
module span_ops {
  pub fn get(s: Span, idx: Int) -> Result[Int, Str] { return span_access(s, idx); }
  pub fn classify(s: Span, idx: Int) -> SpanResult { return classify_span(s, idx); }
  pub fn empty() -> Span { return Span{ id: 0; count: 0; }; }
  pub fn null_ptr() -> *Int { var p: *Int; unsafe { p = 0 as *Int; } return p; }
}
use span_ops.get;
use span_ops.classify;
use span_ops.empty;
use span_ops.null_ptr;
fn main() -> Int {
  var val: Int = 7;
  var s = Span{ id: val; count: 5; };
  var cs = classify(s, 0);
  var chk = 0;
  if cs == SpanResult.Ok { chk += 1; }
  match get(s, 2) { Ok(v) => { if v == 9 { chk += 1; } } Err(_) => {} }
  var bad = classify(s, 10);
  if bad == SpanResult.OutOfBounds(10) { chk += 1; }
  var es = empty();
  if es.count == 0 { chk += 1; }
  var np = null_ptr();
  if np == unsafe { 0 as *Int } { chk += 1; }
  var i = 0;
  while i < 5 { i += 1; }
  if i == 5 { chk += 1; }
  if chk == 6 { return 0; }
  return 1;
}
