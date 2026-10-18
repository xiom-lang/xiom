// m336 residual (C-ORBIT-12): a consumer of the module-qualified alias.
module m336_alias_consumer

use xiom.db.storage.tuple;
use xiom.io;

fn main() -> Int {
  var buf = Vec[Int].new();
  buf.push(7);
  match tuple.read_value(&buf, 0) {
    Err(_) => { io.println("read err"); return 1; }
    Ok(p) => {
      let v = p.0;
      let next = p.1;
      if next != 9 { return 2; }
      io.println("ok");
      return 0;
    }
  }
}
