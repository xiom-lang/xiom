// M36-S07: Codegen -- instruction emission patterns and opcode assignments
type Instr = { opcode: Int; rd: Int; rs1: Int; rs2: Int; imm: Int; }
fn make_instr(op: Int, rd: Int, rs1: Int, rs2: Int, imm: Int) -> Instr {
  return Instr{ opcode: op; rd: rd; rs1: rs1; rs2: rs2; imm: imm; };
}
fn emit_add(rd: Int, rs1: Int, rs2: Int) -> Instr {
  return make_instr(1, rd, rs1, rs2, 0);
}
fn emit_loadi(rd: Int, imm: Int) -> Instr {
  return make_instr(2, rd, 0, 0, imm);
}
fn emit_store(rd: Int, rs1: Int, imm: Int) -> Instr {
  return make_instr(3, rd, rs1, 0, imm);
}
fn emit_mul(rd: Int, rs1: Int, rs2: Int) -> Instr {
  return make_instr(4, rd, rs1, rs2, 0);
}
fn emit_ret(rd: Int) -> Instr {
  return make_instr(99, rd, 0, 0, 0);
}
fn instr_encode(i: Instr) -> Int {
  return i.opcode * 100000 + i.rd * 10000 + i.rs1 * 1000 + i.rs2 * 100 + i.imm;
}
fn main() -> Int {
  var i1 = emit_loadi(1, 42);
  if i1.opcode != 2 || i1.imm != 42 { return 1; }
  var i2 = emit_add(2, 1, 1);
  if i2.opcode != 1 || i2.rd != 2 { return 2; }
  var i3 = emit_mul(3, 2, 2);
  if i3.opcode != 4 { return 3; }
  var i4 = emit_store(3, 0, 100);
  if i4.opcode != 3 || i4.imm != 100 { return 4; }
  var ir = emit_ret(0);
  if ir.opcode != 99 { return 5; }
  var enc = instr_encode(i1);
  if enc != 210042 { return 6; }
  return 0;
}
