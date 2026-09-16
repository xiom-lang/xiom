// XIOM -- stress_generic_5chain
// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

fn id[T](x: T) -> T { return x; }
fn wrap[T](x: T) -> T { return id(x); }
fn double[T](x: T) -> T { return wrap(x); }
fn triple[T](x: T) -> T { return double(x); }
fn quad[T](x: T) -> T { return triple(x); }

fn main() -> Int { return quad(42); }
