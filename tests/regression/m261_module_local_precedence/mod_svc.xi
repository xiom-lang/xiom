// XIOM ORBITDB -- m261 regression fixture: service module with a module-PRIVATE
// zero-arg `arm()` that its public `ping()` calls unqualified. A consumer
// module in the same program defines arm(Str,Str,Str); pre-fix that consumer
// function hijacked this call site through the registration-order bare symbol.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module mod_svc

fn arm() -> Int {
  return 1;
}

pub fn ping() -> Int {
  return arm();
}
