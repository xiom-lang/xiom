// M36-S27: Virtual machine — bytecode instruction dispatch (switch/match simulation)
type Bytecode = { op: Int; operand_a: Int; operand_b: Int; operand_c: Int; }
fn make_bc(op: Int, a: Int, b: Int, c: Int) -> Bytecode {
  return Bytecode{ op: op; operand_a: a; operand_b: b; operand_c: c; };
}
fn exec_nop(_a: Int, _b: Int, _c: Int) -> Int { return 0; }
fn exec_load_const(a: Int, _b: Int, _c: Int) -> Int { return a; }
fn exec_add(a: Int, b: Int, _c: Int) -> Int { return a + b; }
fn exec_sub(a: Int, b: Int, _c: Int) -> Int { return a - b; }
fn exec_mul(a: Int, b: Int, _c: Int) -> Int { return a * b; }
fn exec_div(a: Int, b: Int, _c: Int) -> Int {
  if b == 0 { return 0; }
  return a / b;
}
fn exec_jmp_if(a: Int, b: Int, pc: Int) -> Int {
  if a != 0 { return pc + b; }
  return pc + 1;
}
fn dispatch(bc: Bytecode, reg_a: Int, reg_b: Int, pc: Int) -> Int {
  if bc.op == 0 { return exec_nop(reg_a, reg_b, bc.operand_c); }
  if bc.op == 1 { return exec_load_const(bc.operand_a, 0, 0); }
  if bc.op == 2 { return exec_add(reg_a, reg_b, 0); }
  if bc.op == 3 { return exec_sub(reg_a, reg_b, 0); }
  if bc.op == 4 { return exec_mul(reg_a, reg_b, 0); }
  if bc.op == 5 { return exec_div(reg_a, reg_b, 0); }
  return -1;
}
fn main() -> Int {
  var nop = make_bc(0, 0, 0, 0);
  var load = make_bc(1, 42, 0, 0);
  var add = make_bc(2, 0, 0, 0);
  var sub = make_bc(3, 0, 0, 0);
  var mul = make_bc(4, 0, 0, 0);
  var div = make_bc(5, 0, 0, 0);
  if exec_nop(0, 0, 0) != 0 { return 1; }
  if exec_load_const(42, 0, 0) != 42 { return 2; }
  if exec_add(10, 20, 0) != 30 { return 3; }
  if exec_sub(50, 20, 0) != 30 { return 4; }
  if exec_mul(5, 6, 0) != 30 { return 5; }
  if exec_div(100, 4, 0) != 25 { return 6; }
  if exec_div(10, 0, 0) != 0 { return 7; }
  if exec_jmp_if(1, 5, 10) != 15 { return 8; }
  if exec_jmp_if(0, 5, 10) != 11 { return 9; }
  if dispatch(load, 0, 0, 0) != 42 { return 10; }
  if dispatch(add, 10, 20, 0) != 30 { return 11; }
  if dispatch(nop, 99, 99, 0) != 0 { return 12; }
  return 0;
}
