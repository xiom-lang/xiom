// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// B-003: Option<Str> from method returns with contracts -- regression test
// Verifies Path.file_name() and Path.file_stem() work correctly with contracts enabled.
// NOTE: Do not test Path.new("") -- the stdlib's file_name ensures
//   `result is Some => result.len() > 0` is violated for empty strings
//   (file_name("") returns Some("") with length 0). This is a stdlib bug,
//   not a codegen issue.

use xiom.path;

fn main() -> Int {
  // Test 1: file_name with path
  let p1 = xiom.path.Path.new("/home/user.txt");
  let name = p1.file_name();
  if !name.is_some() { return 1; }
  if name.unwrap() != "user.txt" { return 2; }

  // Test 2: file_name at root returns None
  let p2 = xiom.path.Path.new("/");
  let name2 = p2.file_name();
  if !name2.is_none() { return 3; }

  // Test 3: file_stem extracts stem
  let p3 = xiom.path.Path.new("file.txt");
  let stem = p3.file_stem();
  if !stem.is_some() { return 4; }
  if stem.unwrap() != "file" { return 5; }

  return 0;
}
