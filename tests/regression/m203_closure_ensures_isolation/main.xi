// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m203 (stdlib iter Range.count clause): a `return` inside a closure body must
// not emit the ENCLOSING function's ensures checks / result slot -- the outer
// result alloca register is not defined in the thunk and clang fails with
// "instruction forward referenced with type 'ptr'" (Range.count + ensures:
// result >= 0 blocked smoke_iter).
fn _count_via[T](next: fn() -> Option[T]) -> Int {
  var n = 0;
  var cur = next();
  while cur.is_some {
    n = n + 1;
    cur = next();
  };
  return n;
}

type R = { start: Int; end: Int; }

fn R.next(self) -> Option[Int] {
  if self.start >= self.end { return None; }
  let v = self.start;
  self.start = self.start + 1;
  return Some(v);
}

fn R.count(self) -> Int
  ensures: result >= 0
{
  var r = self;
  return _count_via[Int](fn() -> Option[Int] { return r.next(); });
}

fn main() -> Int {
  var r = R{ start: 0, end: 3 };
  if r.count() != 3 { return 1; }
  return 0;
}
