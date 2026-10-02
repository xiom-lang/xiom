// Phase 3 checker gate, accept case: `Color.Red` parses as a dotted Ident,
// which the Rust diverger/exhaustiveness logic treats as a catch-all binding
// -- so a single-arm enum match produces NO W003/W000.
enum Color { Red, Green, Blue }
fn f(c: Color) -> Int {
  match c {
    Color.Red => { return 1; }
  }
  return 0;
}
