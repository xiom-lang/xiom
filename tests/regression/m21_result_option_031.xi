module m21_result_option_031
type Config = { port: Option[Int]; host: Option[Int]; }

  pub fn run() -> Int {
    var cfg: Config = { port: None; host: None; };
    match cfg.host {
      Some(_) => return 1,
      None => if cfg.port.is_none() { return 0; },
    }
  }
use m21_result_option_031.run;
fn main() -> Int { return run(); }
