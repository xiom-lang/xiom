// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-J05: Module with pub const -- exported constants in modules
module config {
  pub const MAX_USERS: Int = 100;
  pub const RATE: Float64 = 1.5;
  pub const NAME: Str = "app";
  pub const ENABLED: Bool = true;
  pub fn check(v: Int) -> Int { if v <= MAX_USERS { return 0; } return 1; }
}
use config.check;
use config.MAX_USERS;
use config.ENABLED;
fn main() -> Int {
  if MAX_USERS != 100 { return 1; }
  if check(50) != 0 { return 2; }
  if check(200) != 1 { return 3; }
  if ENABLED != true { return 4; }
  return 0;
}
