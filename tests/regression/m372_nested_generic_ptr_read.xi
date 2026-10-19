// m372 -- a generic call nested as the argument of ptr.read was silently
// mis-typed (wrong payload, no diagnostic).
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Expected: rc 0, `nested=42 local=42`.
// Before the fix, `ptr.read(identity_ptr[T](p))` inferred T=Int from the
// pointer argument and read 8 bytes of a 16-byte Pair; staging the generic
// call in a local first (`local_implicit`) was the working shape.
//
// Kept as the compiler-side e2e lock for REL-20261019-1451-stdlib.

module p_nested_generic_ptr_read

use xiom.io;
use xiom.convert;
use xiom.alloc;
use xiom.ptr;
use xiom.core.size_of;

type Pair[T] = { a: Int; b: Int; }

fn identity_ptr[T](p: *Pair[T]) -> *Pair[T]
  requires: p != null
{
  unsafe { return p; }
}

fn nested_implicit[T](p: *Pair[T]) -> Int {
  let q = ptr.read(identity_ptr[T](p));
  return q.a + q.b;
}

fn local_implicit[T](p: *Pair[T]) -> Int {
  let q0 = identity_ptr[T](p);
  let q = ptr.read(q0);
  return q.a + q.b;
}

fn main() -> Int {
  let cell = alloc.alloc(size_of[Pair[Int]]());
  let v = Pair[Int]{ a: 20; b: 22; };
  unsafe { ptr.write(cell as *Pair[Int], v); }
  var p: *Pair[Int];
  unsafe { p = cell as *Pair[Int]; }

  let nested = nested_implicit[Int](p);
  let local = local_implicit[Int](p);
  io.println("nested=" + convert.int_to_string(nested)
    + " local=" + convert.int_to_string(local));

  unsafe { alloc.dealloc(cell, size_of[Pair[Int]]()); }
  if nested != 42 { return 1; }
  if local != 42 { return 2; }
  return 0;
}
