// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-Y13: enum discriminant + generic method dispatch + struct + match + contract + impl + module + diff
type Context = { x: Int; y: Int; }
enum FnKind { A, B, C }
fn invoke[T](ctx: Context, kind: FnKind) -> Int
  requires: ctx.x >= 0
  requires: ctx.y >= 0
  ensures: result >= 0
{
  match kind {
    A => ctx.x + ctx.y,
    B => ctx.x * ctx.y,
    C => if ctx.x > ctx.y { ctx.x } else { ctx.y },
  }
}
fn sum_direct(ctx: Context) -> Int { return ctx.x + ctx.y; }
interface Callable { fn call(self) -> Int; }
impl Callable for Context {
  fn call(self) -> Int { return self.x + self.y; }
}
module dispatch {
  pub fn do_invoke(ctx: Context, k: FnKind) -> Int { return invoke(ctx, k); }
  pub fn do_sum(ctx: Context) -> Int { return sum_direct(ctx); }
  pub fn via_call(ctx: Context) -> Int { return ctx.call(); }
}
use dispatch.do_invoke;
use dispatch.do_sum;
use dispatch.via_call;
enum Route { Invoke, Direct, Call }
fn route(r: Route, ctx: Context, k: FnKind) -> Int {
  match r { Invoke => do_invoke(ctx, k), Direct => do_sum(ctx), Call => via_call(ctx), }
}
fn main() -> Int {
  var ctx = Context{ x: 6; y: 8; };
  var r1 = route(Route.Invoke, ctx, FnKind.A);
  var r2 = route(Route.Direct, ctx, FnKind.A);
  var r3 = route(Route.Call, ctx, FnKind.A);
  if r1 == r2 && r2 == r3 && r1 == 14 { return 0; }
  return 1;
}
