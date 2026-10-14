// m317 fixture module (shape b): a GLOB import of a multi-segment module
// path. Pre-fix the walk stopped on the PARENT map, so the glob imported
// nothing and the bare constant read was T001.
module xiom.m317x.xglobber;

use xiom.m317x.fmod.*;

pub fn glob_get() -> Int {
  return FVALUE;
}
