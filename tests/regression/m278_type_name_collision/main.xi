// m278 (REL-20261011-2137-engine): two modules declare a same-named type
// with different layouts; the entry file's import must decide which one its
// bare `Mesh` references use. Pre-fix the injected bare key was claimed by
// whichever module sorted first (coll.aother) and zuser's make/vlen AV'd on
// the wrong layout -- the engine module-root AV in miniature.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module m278_type_name_collision;

use xiom.io;
use xiom.convert;
use coll.aother;
use coll.zuser;

fn main() -> Int {
  var m = make();
  m.vertices.push(5);
  let n = vlen(&m);
  if n != 1 {
    io.println("  [CORRUPT] vlen=" + convert.int_to_string(n));
    return 1;
  }
  if m.tag2 != 7 {
    return 2;
  }
  if marker() != 1 {
    return 3;
  }
  io.println("m278_type_name_collision: ok");
  return 0;
}
