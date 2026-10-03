// m180 lock (release hold, v0.62.x): xiom.contracts.any_contracts() must not
// crash on a COMPUTED receiver.
//
// `any_contracts()` runs `!_get_index().none()` where `_get_index()` returns
// ContractIndex by value. Codegen could not infer the receiver type of a call
// expression (infer_struct_type_name only walks idents/fields), so the method
// key degraded to the bare leaf "none"; the keep-first catalog alias then
// bound the unrelated generic free fn `core.none[T](items, predicate)`.
// The monomorphised symbol got the free fn's (2-param) signature while the
// call site passed only the receiver -> ABI mismatch -> 0xC0000005.
//
// The method must bind `contracts.ContractIndex.none` and the program must
// exit 0. Needs a stdlib checkout with xiom/core/contracts.xi.
module m180_contracts_any_av

use xiom.contracts;

fn main() -> Int {
  xiom.contracts.any_contracts();
  return 0;
}
