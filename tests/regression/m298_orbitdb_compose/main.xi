// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m298 lock (REL-20261012-1956-orbitdb): two co-imported modules declare the
// same public type triad (BTree / NodeStore / BTreeNode). The second
// module's BTree was shape-identical to the first's, so the collision triage
// exempted it as a "facade duplicate": its declaration was dropped
// (first-wins bare key) and its bodies compiled against the FIRST module's
// BTree meta, whose `store` field had been rewritten to the first module's
// NodeStore. `child.payloads.push/pop` then resolved the method against the
// first module's BTreeNode (no payloads field) -> codegen C001
// 'xiom.aa.btree.BTreeNode.push'/'pop' -- the xiom.btree +
// xiom.db.engine compose collision in miniature. Pre-fix binary: compile
// fails (C001); post-fix both triads keep module identity.
module m298_orbitdb_compose;

use xiom.io;
use xiom.convert;
use xiom.aa.btree;
use xiom.zz.engine;

fn main() -> Int {
  var a = atree_new(2);
  let ra = atree_bump(&mut a);
  if ra != 1 {
    io.println("m298_orbitdb_compose: [CORRUPT] aa bump=" + convert.int_to_string(ra));
    return 1;
  }

  var e = engine_new(3);
  let re = engine_insert(&mut e, 7);
  if re != 1 {
    io.println("m298_orbitdb_compose: [CORRUPT] engine insert=" + convert.int_to_string(re));
    return 2;
  }
  let rt = engine_trim(&mut e);
  if rt != 0 {
    io.println("m298_orbitdb_compose: [CORRUPT] engine trim=" + convert.int_to_string(rt));
    return 3;
  }

  io.println("m298_orbitdb_compose: ok");
  return 0;
}
