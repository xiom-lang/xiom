// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m353 lock (REL-20261011-1456-stdlib), hermetic half: a module-declared
// generic `Mutex[T]` with `new`/`get`/`lock`/`unlock` methods. The compiler
// registers builtin `Mutex.new/lock/unlock/destroy` signatures globally (the
// v0.56 I3 barrier); pre-fix those won the bare `Type.method` lookup, so
// `mutex.Mutex.new(7)` failed T001 ("'new' expects 0 argument(s), found 1")
// and no stdlib/user declaration with the same leaf was reachable. Post-fix
// the declared receiver methods own every call shape.
module xiom.m353x.mutex;

pub type Mutex[T] = {
  value: T;
}

pub fn Mutex.new[T](value: T) -> Mutex[T] {
  return Mutex[T] { value: value };
}

pub fn Mutex.get[T](self) -> T {
  return self.value;
}

pub fn Mutex.lock[T](self) -> T {
  return self.value;
}

pub fn Mutex.unlock[T](self) -> Bool {
  return true;
}
