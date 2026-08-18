// BUG 51 regression: Option[UserStruct] payloads — the Some-bound name must
// (a) type-check method calls against the payload's methods (the `_` wildcard
// fell to the sorted wildcard lookup: Option.get before MyRc.get -> "cannot
// compare Option with Int") and (b) read the correct payload at runtime.
module m37_bug51_option_struct_payload
use xiom.core;

type MyRc = {
  value: Int;
}

fn MyRc.get(self) -> Int {
  return self.value;
}

fn try_get() -> Option[MyRc] {
  return Some(MyRc{ value: 42; });
}

fn main() -> Int {
  match try_get() {
    Some(up) => {
      let direct = up.get();
      var x: MyRc = up;
      let via_x = x.get();
      if direct != 42 { return 1; }
      if via_x != 42 { return 2; }
      return 0;
    }
    None => { return 3; }
  }
}
