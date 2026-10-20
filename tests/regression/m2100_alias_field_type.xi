// m2100-W1 (REL-20261016-2100-stdlib): alias-qualified and full-path module
// types in a STRUCT FIELD position lost method dispatch.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// `{ stream: net.TcpStream; }` / `{ stream: xiom.net.TcpStream; }` then
// `h.stream.close()` emitted `unknown type 'net.TcpStream' -- defaulting to
// i64` (v0.64.5) and later C001 `close` unresolved (v0.64.6-v0.64.8): the
// receiver kept the raw qualified spelling while the registered type key was
// leaf/xiom-prefixed. Exact -> xiom-prefixed -> deterministic leaf key
// normalization now matches the bare-leaf control.

module m2100_alias_field_type

use xiom.io;
use xiom.net;

type HolderAlias = { stream: net.TcpStream; open: Bool; }

type HolderFull = { stream: xiom.net.TcpStream; open: Bool; }

type HolderBare = { stream: TcpStream; open: Bool; }

fn main() -> Int {
  var a = HolderAlias{ stream: net.TcpStream{ fd: 0 - 1 }; open: false; };
  if a.open { return 1; }
  let _ = a.stream.close();

  var b = HolderFull{ stream: net.TcpStream{ fd: 0 - 1 }; open: false; };
  if b.open { return 2; }
  let _ = b.stream.close();

  var c = HolderBare{ stream: net.TcpStream{ fd: 0 - 1 }; open: false; };
  if c.open { return 3; }
  let _ = c.stream.close();

  io.println("field_type_ok");
  return 0;
}
