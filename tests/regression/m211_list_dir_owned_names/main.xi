// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m211 (stdlib io.list_dir relay): `Str::from_c_str` must produce an OWNED
// Str. Pre-fix it was a pointer identity, so `list_dir` pushed aliases of
// readdir's reused dirent storage: the correct count with the LAST name
// repeated for every entry. Directory entry names are unique, so exactly one
// element may equal the first.
use xiom.io;

fn main() -> Int {
  let dir = "m211_listdir_probe";
  let _ = io.create_dir(dir);
  if !io.is_dir(dir) { return 4; }
  let _ = io.write_file(dir + "/alpha.txt", "a");
  let _ = io.write_file(dir + "/beta.txt", "b");
  let _ = io.write_file(dir + "/gamma.txt", "c");
  let r = io.list_dir(dir);
  match r {
    Ok(names) => {
      if names.len() != 3 { return 1; }
      var i = 0;
      var first_count = 0;
      while i < names.len() {
        if names[i] == names[0] { first_count = first_count + 1; }
        i = i + 1;
      };
      if first_count != 1 { return 2; }
    },
    Err(e) => { return 3; },
  }
  return 0;
}
