// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-E08: Nested if blocks 10 deep -- control flow nesting stress
fn main() -> Int {
  var depth = 0;
  if true {
    depth = depth + 1;
    if true {
      depth = depth + 1;
      if true {
        depth = depth + 1;
        if true {
          depth = depth + 1;
          if true {
            depth = depth + 1;
            if true {
              depth = depth + 1;
              if true {
                depth = depth + 1;
                if true {
                  depth = depth + 1;
                  if true {
                    depth = depth + 1;
                    if true {
                      depth = depth + 1;
                    }
                  }
                }
              }
            }
          }
        }
      }
    }
  }
  if depth != 10 { return 1; }
  return 0;
}