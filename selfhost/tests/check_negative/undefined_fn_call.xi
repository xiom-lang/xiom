// Phase 3 checker gate, negative case: unresolved call target reports the
// undefined identifier and then the observed cascade (the unresolved call
// types as Unit, so the return check reports `found ()`).
fn main() -> Int {
  return no_such_fn(1);
}
