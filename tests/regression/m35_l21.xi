// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-L21: Array of struct with pointers -- verify pointer semantics in struct arrays
type Cell = { data: Int; next_id: Int; }

fn get_data(c: Cell) -> Int {
  return c.data;
}

fn get_next_id(c: Cell) -> Int {
  return c.next_id;
}

fn main() -> Int {
  var arr = [Cell{ data: 11; next_id: 1; }, Cell{ data: 22; next_id: 2; }, Cell{ data: 33; next_id: 3; }];
  if get_data(arr[0]) != 11 { return 1; }
  if get_data(arr[1]) != 22 { return 2; }
  if get_data(arr[2]) != 33 { return 3; }
  if get_next_id(arr[0]) != 1 { return 4; }
  if get_next_id(arr[2]) != 3 { return 5; }
  return 0;
}
