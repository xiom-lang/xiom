// Phase 3 checker gate, negative case: a bare identifier that resolves to
// nothing (no `use`, so the permissive catalog path stays off).
fn main() -> Int {
  let x = missing_name;
  return 0;
}
