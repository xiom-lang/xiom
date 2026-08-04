// E2E: Send enforcement — nested struct with all-Send fields
use xiom.io;

type Inner = { x: Int; y: Int; }
type Outer = { label: Str; pos: Inner; }

fn main() -> Int {
  var o = Outer{ label: "test", pos: Inner{ x: 10, y: 20 } };
  spawn move {
    if o.pos.x == 10 { io.println("PASS: nested struct is Send"); }
  }
  return 0;
}
