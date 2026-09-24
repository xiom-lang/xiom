// Relay lock: m134.alpha's PRIVATE Timer (1 field). The same-leaf triage used
// to ignore private types, so alpha's Timer and beta's Timer collapsed into
// one bare `%struct.Timer` (first module won) and the loser silently reused
// the wrong layout.
module m134.alpha

type Timer = {
  deadline: Int;
}

pub fn make() -> Int {
  var t = Timer{ deadline: 1 };
  return t.deadline;
}
