// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m74 (R21): a user `use X as Y;` alias must shadow a catalog module whose
// LEAF equals Y. `use network as net;` used to be a silent no-op and the
// follow-up `use net.local;` then catalog-loaded xiom.net, pulling the whole
// net graph into the compile. Under the strict catalog-finding flip those
// unrelated bodies hard-failed the program (encoding.xi's bare `Vec.get`
// resolved to core's Box.get when collections was not loaded).
module network {
  pub fn ping() -> Int { return 200; }
  pub fn timeout() -> Int { return 408; }
  pub type Addr = { host: Str; port: Int; }
  pub fn local() -> Addr { return Addr{ host: "127.0.0.1"; port: 8080; }; }
}
use network as net;
use net.ping;
use net.timeout;
use net.local;
fn main() -> Int {
  if ping() != 200 { return 1; }
  if timeout() != 408 { return 2; }
  var a = local();
  if a.port != 8080 { return 3; }
  if net.ping() != 200 { return 4; }
  if net.local().port != 8080 { return 5; }
  return 0;
}
