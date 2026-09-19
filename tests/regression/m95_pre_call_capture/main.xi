module p_pre_call_capture

// R49 lock (stdlib relay p_pre_call_capture): @pre on a CALL reads the
// ENTRY state -- the snapshot must be deep enough that a Vec field's
// buffer is cloned, and ref params are rebound through a pointer slot.

type Box = {
  v: Vec[Int];
}

fn total(b: &Box) -> Int {
  return b.v[0];
}

fn bump(b: &mut Box)
  ensures: total(b) == total(b)@pre + 1
{
  b.v[0] = b.v[0] + 1;
}

fn main() -> Int {
  var b = Box{ v: Vec[Int].new() };
  b.v.push(0);
  bump(&mut b);
  return 0;
}