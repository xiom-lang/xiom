// LET-array decision P1 regression (docs/LET_ARRAY_DECISION.md): an
// ANNOTATED fixed array `let c: [N]T = [literals]` binds `[N x T]` (same as
// `var`), elements stored directly into the aggregate slot. Pre-fix the
// M33 Vec conversion fed a %struct.Vec/i8* value into the declared
// [3 x i64] slot: clang rejected the store ("defined with type 'i64/pt' but
// expected '[3 x i64]'"). Unannotated let stays on the Vec bridge.
module m67_let_array_annotated
use xiom.array;

fn main() -> Int {
  let c: [3]Int = [7, 8, 9];
  if c[0] != 7 { return 1; }
  if c[2] != 9 { return 2; }
  let n: [3]Int8 = [1 as Int8, 2 as Int8, 3 as Int8];
  if n[0] != 1 { return 3; }
  if n[2] != 3 { return 4; }
  let f: [2]Float64 = [1.5, 2.5];
  if f[1] != 2.5 { return 5; }
  var v: [3]Int = [4, 5, 6];
  v[1] = 50;
  if v[1] != 50 { return 6; }
  let u = [1, 2, 3, 4];
  if array.len(&u) != 4 { return 7; }
  if u[3] != 4 { return 8; }
  return 0;
}
