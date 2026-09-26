// m144 (sibling-method receiver binding): a receiver-qualified fn whose body
// only calls SIBLING methods bare (`insert(k, v)` in HashMap.resize, `get(k)`
// in HashMap.contains, `get(k)` in the Holder chain) must still get the
// %param_self slot and resolve those bare calls via G-10. The old flow:
//   * monomorphised generic bodies never set the receiver context, so G-10
//     was dead there;
//   * bare sibling calls to generic methods mapped the explicit args
//     positionally to (self, ...) -- the KEY was inttoptr'd as the receiver
//     pointer and dropped from the call (2-arg call against a 3-param
//     definition -> access violation);
//   * body_uses_receiver_state did not count bare sibling calls, so such
//     methods were registered WITHOUT a receiver slot while the call site
//     still passed one (arg shift -> AV);
//   * the G-10 receiver check compared `%struct.X*` against `%struct.X`
//     with ends_with (false -- the string ends with '*'), silently dropping
//     the receiver for non-generic this-based sibling calls.
module m144_sibling_method_calls;

use xiom.collections;

pub type Holder = { m: Map[Int, Int]; }

// Non-generic this-based siblings: `get` is called BARE from `contains`.
pub fn Holder.get(k: &Int) -> Option[Int] { return m.get(k); }
pub fn Holder.contains(k: &Int) -> Bool {
  match get(k) {
    Some(_) => true,
    None => false,
  }
}

fn main() -> Int {
  var h = Holder{ m: Map[Int, Int].new() };
  h.m.insert(1, 5);
  if h.contains(&1) == false { return 2; }
  if h.contains(&2) == true { return 3; }

  // Generic this-based siblings plus the HashMap.resize re-insert loop:
  // 20 inserts force a resize at len 13 (len*4 > cap*3), whose re-insert
  // loop calls `insert(k, v)` bare.
  var hm = HashMap[Int, Int].new();
  var i = 0;
  while i < 20 {
    hm.insert(i, i * 10);
    i = i + 1;
  }
  if hm.count() != 20 { return 4; }

  i = 0;
  while i < 20 {
    if hm.contains(&i) == false { return 5; }
    match hm.get(&i) {
      Some(v) => { if v != i * 10 { return 6; } }
      None => { return 7; }
    }
    i = i + 1;
  }

  if hm.contains(&99) == true { return 8; }
  match hm.get(&99) {
    Some(_) => { return 9; }
    None => { }
  }
  return 0;
}
