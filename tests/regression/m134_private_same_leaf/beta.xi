// Relay lock: m134.beta's PRIVATE Timer (2 fields) -- same leaf as
// m134.alpha.Timer but a different layout.
module m134.beta

type Timer = {
  deadline: Int;
  armed: Bool;
}

pub fn use_it() -> Int {
  var t = Timer{ deadline: 2; armed: true };
  if t.armed { return t.deadline; }
  return 0;
}
