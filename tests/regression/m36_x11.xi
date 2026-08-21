// M36-X11: --emit-ir patterns -- structures that produce interesting IR
enum ExprKind { Const, Add, Sub, Mul, Div, Neg }
type Expr = { kind: ExprKind; left: Int; right: Int; value: Int; }
fn eval_expr(e: Expr) -> Int {
  match e.kind {
    ExprKind.Const => e.value,
    ExprKind.Neg => 0 - e.value,
    ExprKind.Add => e.value,
    ExprKind.Sub => e.value,
    ExprKind.Mul => e.value,
    ExprKind.Div => e.value,
  }
}
fn build_const(v: Int) -> Expr {
  return Expr{ kind: ExprKind.Const; left: 0; right: 0; value: v; };
}
fn build_binop(kind: ExprKind, a: Int, b: Int) -> Expr {
  var v: Int = 0;
  if kind == ExprKind.Add { v = a + b; }
  if kind == ExprKind.Mul { v = a * b; }
  if kind == ExprKind.Sub { v = a - b; }
  if kind == ExprKind.Div { v = a / b; }
  return Expr{ kind: kind; left: a; right: b; value: v; };
}
fn main() -> Int {
  var e1 = build_binop(ExprKind.Add, 2, 3);
  if eval_expr(e1) != 5 { return 1; }
  var e2 = build_binop(ExprKind.Mul, 4, 5);
  if eval_expr(e2) != 20 { return 2; }
  var e3 = build_const(42);
  if eval_expr(e3) != 42 { return 3; }
  var e4 = build_binop(ExprKind.Sub, 10, 3);
  if eval_expr(e4) != 7 { return 4; }
  var e5 = build_binop(ExprKind.Div, 20, 4);
  if eval_expr(e5) != 5 { return 5; }
  return 0;
}
