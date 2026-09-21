// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-J04: Module with pub enum -- public enum types in modules
module result {
  pub enum Status { Success, Failure(code: Int), Pending }
  pub fn ok() -> Status { return Status.Success; }
  pub fn fail(c: Int) -> Status { return Status.Failure(c); }
  pub fn wait() -> Status { return Status.Pending; }
}
use result.Status;
use result.ok;
use result.fail;
use result.wait;
fn check(s: Status) -> Int {
  match s {
    Status.Success => return 1,
    Status.Failure(c) => return c,
    Status.Pending => return -1,
  }
}
fn main() -> Int {
  if check(ok()) != 1 { return 1; }
  if check(fail(99)) != 99 { return 2; }
  if check(wait()) != -1 { return 3; }
  return 0;
}
