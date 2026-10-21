module repro.bom_impl

// The first byte above is U+FEFF (EF BB BF). Do not strip it: this fixture
// locks C-ORBIT-15, where a BOM made the catalog header scan skip this module
// and the import failed with a bare T001 unresolved-import diagnostic.
pub fn bom_value() -> Int {
  return 7;
}
