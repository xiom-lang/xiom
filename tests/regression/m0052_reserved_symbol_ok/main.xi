module m0052_reserved_symbol_ok
fn recv_bytes(n: Int) -> Int {
  return n + 1;
}
fn main() -> Int {
  return recv_bytes(0);
}
