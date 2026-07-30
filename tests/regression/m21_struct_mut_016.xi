module m21_struct_mut_016
type Slot = { value: Int; active: Bool; }
fn main() -> Int {
  var s: Slot = Slot{ value: 0; active: false; };
  s.value = 5;
  s.active = true;
  var new_s: Slot = Slot{ value: s.value + 10; active: false; };
  if new_s.value == 15 && !new_s.active { return 0; }
  return 1;
}
