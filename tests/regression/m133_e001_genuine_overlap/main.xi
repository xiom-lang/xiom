// m133 (E001 genuine overlap): an immutable borrow that is still LIVE when
// the same place is mutated MUST keep warning -- the conservatism fix only
// releases temporaries, never bound borrows.
module m133_e001_genuine_overlap;

fn take_mut(x: &mut Int) { *x = *x + 1; }

fn main() -> Int {
  var m = 1;
  let r = &m;
  take_mut(&mut m);
  if *r != 2 { return 1; }
  return 0;
}
