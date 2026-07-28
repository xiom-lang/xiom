// M36-S30: Linker/loader — ELF/PE header simulation and symbol resolution
type ElfIdent = { magic0: Int; magic1: Int; magic2: Int; magic3: Int; class: Int; endian: Int; }
type ElfHeader = { ident: ElfIdent; e_type: Int; e_machine: Int; e_entry: Int; }
type Symbol = { name: Str; addr: Int; size: Int; binding: Int; }
fn make_ident(m0: Int, m1: Int, m2: Int, m3: Int, cls: Int, endian: Int) -> ElfIdent {
  return ElfIdent{ magic0: m0; magic1: m1; magic2: m2; magic3: m3; class: cls; endian: endian; };
}
fn make_elf_header(ident: ElfIdent, etype: Int, machine: Int, entry: Int) -> ElfHeader {
  return ElfHeader{ ident: ident; e_type: etype; e_machine: machine; e_entry: entry; };
}
fn make_symbol(name: Str, addr: Int, size: Int, binding: Int) -> Symbol {
  return Symbol{ name: name; addr: addr; size: size; binding: binding; };
}
fn is_valid_elf(ident: ElfIdent) -> Bool {
  return ident.magic0 == 0x7F && ident.magic1 == 69 && ident.magic2 == 76 && ident.magic3 == 70;
}
fn is_64bit_elf(ident: ElfIdent) -> Bool { return ident.class == 2; }
fn is_little_endian_elf(ident: ElfIdent) -> Bool { return ident.endian == 1; }
fn is_exe_type(hdr: ElfHeader) -> Bool { return hdr.e_type == 2; }
fn is_dyn_type(hdr: ElfHeader) -> Bool { return hdr.e_type == 3; }
fn is_x86_64(hdr: ElfHeader) -> Bool { return hdr.e_machine == 62; }
fn is_global_sym(s: Symbol) -> Bool { return s.binding == 1; }
fn is_local_sym(s: Symbol) -> Bool { return s.binding == 0; }
fn main() -> Int {
  var ident = make_ident(0x7F, 69, 76, 70, 2, 1);
  if !is_valid_elf(ident) { return 1; }
  var bad_ident = make_ident(0, 0, 0, 0, 2, 1);
  if is_valid_elf(bad_ident) { return 2; }
  if !is_64bit_elf(ident) { return 3; }
  if !is_little_endian_elf(ident) { return 4; }
  var hdr = make_elf_header(ident, 2, 62, 0x400000);
  if !is_exe_type(hdr) { return 5; }
  if is_dyn_type(hdr) { return 6; }
  if !is_x86_64(hdr) { return 7; }
  var sym_main = make_symbol("main", 0x401000, 128, 1);
  var sym_local = make_symbol("helper", 0x402000, 64, 0);
  if !is_global_sym(sym_main) { return 8; }
  if is_global_sym(sym_local) { return 9; }
  if !is_local_sym(sym_local) { return 10; }
  if is_local_sym(sym_main) { return 11; }
  if sym_main.size != 128 || sym_main.addr != 0x401000 { return 12; }
  return 0;
}
