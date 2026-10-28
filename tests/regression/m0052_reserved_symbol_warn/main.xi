module m0052_reserved_symbol_warn
fn recv(n: Int) -> Int {
  return n + 1;
}
fn main() -> Int {
  return recv(0);
}
