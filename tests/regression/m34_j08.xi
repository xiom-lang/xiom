// M34-J08: use module as alias — module alias imports
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
  return 0;
}
