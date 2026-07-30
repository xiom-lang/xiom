module m21_struct_mut_017
type Entry = { key: Int; val: Int; }
fn main() -> Int {
  var res: Result[Entry, Int] = Ok({ key: 1; val: 100; });
  match res {
    Ok(e) => {
      if e.key == 1 && e.val == 100 { return 0; }
    }
    Err(_) => { return 1; }
  }
  return 1;
  return 1;
}
