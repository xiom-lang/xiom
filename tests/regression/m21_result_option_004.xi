module m21_result_option_004
type Payload = { id: Int; data: Int; }

  pub fn run() -> Int {
    var r: Result[Payload, Int] = Ok({ id: 7; data: 99; });
    match r {
      Ok(p) => if p.id == 7 && p.data == 99 { return 0; },
      Err(_) => return 1,
    }
  }
use m21_result_option_004.run;
fn main() -> Int { return run(); }
