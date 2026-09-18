// R47 lock (playground C18/C19): conversion methods and Option payloads.
//
// (1) `x.to_str()` on builtin types must reach the concrete xiom.fmt
//     conversion even when the program never `use`d xiom.fmt. Before R47 the
//     checker accepted the call but codegen emitted a zero-arg i64 stub
//     (Str printed empty; the playground pinned toolchain printed Float64
//     IEEE bit patterns as decimals).
// (2) `Option[Str].unwrap_or("...")` used to emit the default's `ptrtoint`
//     in the ok block while the phi tagged it on the fail edge -- clang
//     rejected the IR ("Instruction does not dominate all uses"), and the
//     i64 result was then truncated to a single byte at the call site.
// (3) Chained receivers (`o.unwrap_or("x").to_str()`, `v[0].to_str()`) have
//     no declared return type; they used to lower to xiom_int_to_string and
//     print pointer bits.
module m91_conversion_methods

use xiom.io;

type Person = {
  name: Str;
  age: Int;
}

fn main() -> Int {
  // (1) builtin receivers, no `use xiom.fmt`.
  let s: Str = "plain";
  if s.to_str() != "plain" { return 1; }
  let n: Int = 42;
  if n.to_str() != "42" { return 2; }
  let f: Float64 = 1.5;
  if f.to_str() != "1.5" { return 3; }
  let b: Bool = true;
  if b.to_str() != "true" { return 4; }

  // Struct field Str.
  let p = Person { name: "Ada", age: 36 };
  if p.name.to_str() != "Ada" { return 5; }

  // (2)+(3) Option payload default and chained conversion.
  let o: Option[Str] = Some("inside");
  if o.unwrap_or("none") != "inside" { return 6; }
  if o.unwrap_or("none").to_str() != "inside" { return 7; }
  let missing: Option[Str] = None;
  if missing.unwrap_or("fallback") != "fallback" { return 8; }

  let of: Option[Float64] = Some(2.5);
  if of.unwrap_or(0.0).to_str() != "2.5" { return 9; }

  // Vec element Str handle.
  let v: Vec[Str] = ["alpha", "beta"];
  if v[0].to_str() != "alpha" { return 10; }

  io.println(o.unwrap_or("none"));
  io.println(v[1].to_str());
  return 0;
}
