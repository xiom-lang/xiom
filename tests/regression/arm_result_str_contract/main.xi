module arm_result_str_contract

use xiom.io;

fn main() -> Int {
  let r = io.read_file("no-such-file.txt");
  match r {
    Ok(s) => { println(s); }
    Err(e) => { println(e.message); }
  }
  return 0;
}
