// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-J09: Dotted path: module.submodule.fn() -- deep module paths
module root {
  pub fn head() -> Int { return 1; }
  module branch {
    pub fn level() -> Int { return 10; }
    module leaf {
      pub fn tip() -> Int { return 100; }
    }
  }
}
use root.head;
use root.branch.level;
use root.branch.leaf.tip;
fn main() -> Int {
  var r1 = head();
  var r2 = level();
  var r3 = tip();
  var r4 = root.head();
  var r5 = root.branch.level();
  var r6 = root.branch.leaf.tip();
  if r1 == 1 && r2 == 10 && r3 == 100 { return 0; }
  if r4 == 1 && r5 == 10 && r6 == 100 { return 0; }
  return 1;
}
