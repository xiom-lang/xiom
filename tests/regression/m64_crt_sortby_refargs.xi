// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M64 (CRT-layout family #2): `&mut [N]T` param element-ADDRESS lowering.
// `&arr[j-1]` inside a fn taking `&mut [N]T` compiled the element VALUE as
// the address (the Ref arm's by-value fixed-array branch requires a
// `[N x T]` slot; the param's slot is the bare data pointer `i64*`), so a
// fn-typed comparator derefed small ints -> 0xC0000005 (smoke_array_sort_by
// startup AV). Fixes: (1) MutRef array params now register their element
// type (local_array_elem) like Ref ones; (2) the Ref arm emits GEP +
// ptrtoint for pointer-typed array params (is_array_elem_param).
module m64_crt_sortby_refargs

use xiom.array;
use xiom.cmp;

fn main() -> Int {
  var arr = [5, 3, 1, 4, 2];
  array.sort_by(&mut arr, fn(a: &Int, b: &Int) -> cmp.Ordering {
    if *a < *b { return cmp.Less; };
    if *a > *b { return cmp.Greater; };
    return cmp.Equal;
  });
  if arr[0] != 1 { return 1; }
  if arr[1] != 2 { return 2; }
  if arr[2] != 3 { return 3; }
  if arr[3] != 4 { return 4; }
  if arr[4] != 5 { return 5; }
  return 0;
}
