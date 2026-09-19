module m99_module_path_alias

// R49-1 lock (stdlib relay p_module_path_alias): importing a moved module by
// its FILE PATH (`xiom.crypto.legacy.md5`, declaring `xiom.crypto.md5`) must
// register the DECLARED identity -- the path alias used to corrupt
// xiom.crypto's exports (33 T001s inside rng_crypto).

use xiom.crypto.legacy.md5;

fn main() -> Int {
  return 0;
}