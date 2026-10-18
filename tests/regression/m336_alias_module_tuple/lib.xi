// m336 residual (C-ORBIT-12): module-qualified generic alias + tuple.
module xiom.db.storage.tuple

enum DbError { Corruption }

pub type DbResult[T] = Result[T, DbError];

pub type TValue = enum {
  VNull,
  VInt(x: Int),
  VFloat(f: Float64),
  VText(s: Str),
  VBlob(b: Vec[Int]),
}

pub fn read_value(buf: &Vec[Int], at: Int) -> DbResult[(TValue, Int)] {
  if at < 0 { return Err(DbError.Corruption); }
  if at >= buf.len() { return Err(DbError.Corruption); }
  return Ok((TValue.VInt(buf[at]), at + 9));
}
