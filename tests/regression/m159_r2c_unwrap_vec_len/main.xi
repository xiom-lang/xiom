// m159 (R-2c): `.len()` on an inline `opt.unwrap()` receiver must resolve
// the Vec payload -- both the annotated and the unannotated Option[Vec[Int]]
// shapes used to route it to `xiom_str_len` (annotated: clang error passing
// %struct.Vec; unannotated: strlen of the boxed handle -> 6). Str payloads
// must keep the Str.len path.
module m159_r2c_unwrap_vec_len;

use xiom.io;

fn main() -> Int {
  var opt: Option[Vec[Int]] = Some(Vec[Int].new());
  match opt {
    Some(v) => { v.push(7); }
    None => { return 1; }
  }
  let n: Int = opt.unwrap().len();
  if n != 1 { return 2; }

  let opt2 = Some(Vec[Int].new());
  let m: Int = opt2.unwrap().len();
  if m != 0 { return 3; }

  let so: Option[Str] = Some("hi");
  let sl: Int = so.unwrap().len();
  if sl != 2 { return 4; }

  io.println(n.to_str());
  return 0;
}
