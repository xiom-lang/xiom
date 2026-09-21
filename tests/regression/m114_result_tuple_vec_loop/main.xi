// m114 (R59, stdlib p_result_tuple_vec_loop): the enclosing match arm's
// result slot leaked into loop bodies -- `oid.push(value)` as the while's last
// expression stored a %struct.Vec into the arm's %struct.Result slot
// ("%tmp157 ... type %struct.Vec ... but expected %struct.Result"). Only the
// DIRECT arm body may store into the match result slot; While/For/Spawn bodies
// are statement contexts now.
module m114.main

pub fn f(data: &Vec[UInt8]) -> Result[(Vec[Int], Int), Str] {
  let tlv: Result[(Int, Int, Int), Str] = Ok((0x06, 0, 2));
  match tlv {
    Err(e) => Err(e);
    Ok(t) => {
      let end = t.2;
      var oid = Vec[Int].new();
      var i = 0;
      while i < end {
        var value: Int = 0;
        let b = data[i] as Int;
        value = value * 128 + (b & 0x7F);
        i = i + 1;
        oid.push(value);
      }
      Ok((oid, end));
    }
  }
}

fn main() -> Int {
  var v: Vec[UInt8] = Vec[UInt8].new();
  v.push(1 as UInt8);
  v.push(2 as UInt8);
  let r = f(&v);
  return 0;
}
