// m118 (R65): `xiom.env.OS/ARCH/FAMILY` are compile-time TARGET facts and must
// agree with the running host. They were hardcoded to windows/x86_64 in the
// stdlib, so a Linux build reported windows (stdlib p_platform_env: env.OS
// = "windows" while os.platform.is_linux = true). The compiler now derives
// them from the target triple / host.
module m118_env_platform_constants

use xiom.io;
use xiom.os.env;

fn main() -> Int {
  let is_windows = match env.var_opt("OS") {
    Some(v) => v == "Windows_NT",
    None => false,
  };
  if is_windows {
    if xiom.env.OS != "windows" { return 1; }
    if xiom.env.FAMILY != "windows" { return 2; }
    if xiom.env.ARCH != "x86_64" && xiom.env.ARCH != "aarch64" { return 3; }
  } else {
    if xiom.env.OS == "windows" { return 4; }
    if xiom.env.FAMILY == "windows" { return 5; }
    if xiom.env.FAMILY != "unix" { return 6; }
    if xiom.env.OS != "linux" && xiom.env.OS != "macos" { return 7; }
  }
  io.println("env=" + xiom.env.OS + "/" + xiom.env.FAMILY + "/" + xiom.env.ARCH);
  return 0;
}
