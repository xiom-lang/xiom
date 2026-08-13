module repro_timer_literal

type Timer = {
  deadline: Int;
  armed: Bool;
  label: Str;
}

pub fn make_timer(dl: Int) -> Timer {
  return Timer{ deadline: dl; armed: true; label: "t"; };
}

pub fn read_fields(t: &Timer) -> Int {
  if t.deadline != 42 { return 1; }
  if !t.armed { return 2; }
  if t.label != "t" { return 3; }
  return 0;
}
