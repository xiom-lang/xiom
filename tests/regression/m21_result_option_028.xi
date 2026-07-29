module m21_result_option_028
type Response = { status: Result[Int, Int]; body: Int; }

  pub fn run() -> Int {
    var resp: Response = { status: Ok(200); body: 42; };
    match resp.status {
      Ok(code) => if code == 200 && resp.body == 42 { return 0; },
      Err(_) => return 1,
    }
  }
use m21_result_option_028.run;
fn main() -> Int { return run(); }
