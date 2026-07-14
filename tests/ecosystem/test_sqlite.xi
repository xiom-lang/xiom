// XIOM — Ecosystem SQLite Type Hardening Tests
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.
//
// Self-contained SQLite type definitions and tests. Exercises enums
// with data, string generation from schema definitions, and Vec ops.

module tests.ecosystem.test_sqlite
use xiom.collections;

pub enum SqliteValue {
  Null,
  Integer(val: Int),
  Real(val: Float64),
  Text(val: Str),
  Blob(val: Vec[UInt8]),
}

pub type SqliteRow = {
  values: Vec[SqliteValue];
}

pub type SqliteResult = {
  columns: Vec[Str];
  rows: Vec[SqliteRow];
}

pub enum SqliteAffinity {
  Text,
  Numeric,
  Integer,
  Real,
  Blob,
}

pub type ColumnDef = {
  name: Str;
  affinity: SqliteAffinity;
  nullable: Bool;
  primary_key: Bool;
}

pub type TableDef = {
  name: Str;
  columns: Vec[ColumnDef];
}

fn SqliteRow.new() -> SqliteRow {
  var values = Vec[SqliteValue].new();
  return SqliteRow{ values: values };
}

fn SqliteRow.add_value(row: &mut SqliteRow, val: SqliteValue) {
  row.values.push(val);
}

fn SqliteRow.get_value(row: &SqliteRow, index: Int) -> Option[SqliteValue] {
  if index < 0 { return None; }
  if index >= row.values.len() { return None; }
  return Some(row.values[index]);
}

fn SqliteRow.len(row: &SqliteRow) -> Int {
  return row.values.len();
}

fn SqliteValue.is_null(v: &SqliteValue) -> Bool {
  match v {
    SqliteValue.Null => { return true; }
    _ => { return false; }
  }
}

fn SqliteValue.is_integer(v: &SqliteValue) -> Bool {
  match v {
    SqliteValue.Integer(_) => { return true; }
    _ => { return false; }
  }
}

fn SqliteValue.is_real(v: &SqliteValue) -> Bool {
  match v {
    SqliteValue.Real(_) => { return true; }
    _ => { return false; }
  }
}

fn SqliteValue.is_text(v: &SqliteValue) -> Bool {
  match v {
    SqliteValue.Text(_) => { return true; }
    _ => { return false; }
  }
}

fn SqliteValue.is_blob(v: &SqliteValue) -> Bool {
  match v {
    SqliteValue.Blob(_) => { return true; }
    _ => { return false; }
  }
}

fn SqliteValue.as_int(v: &SqliteValue) -> Option[Int] {
  match v {
    SqliteValue.Integer(val) => { return Some(val); }
    _ => { return None; }
  }
}

fn SqliteValue.as_real(v: &SqliteValue) -> Option[Float64] {
  match v {
    SqliteValue.Real(val) => { return Some(val); }
    _ => { return None; }
  }
}

fn SqliteValue.as_text(v: &SqliteValue) -> Option[Str] {
  match v {
    SqliteValue.Text(val) => { return Some(val); }
    _ => { return None; }
  }
}

fn SqliteResult.new() -> SqliteResult {
  var columns = Vec[Str].new();
  var rows = Vec[SqliteRow].new();
  return SqliteResult{ columns: columns, rows: rows };
}

fn SqliteResult.add_column(r: &mut SqliteResult, name: Str) {
  r.columns.push(name);
}

fn SqliteResult.add_row(r: &mut SqliteResult, row: SqliteRow) {
  r.rows.push(row);
}

fn SqliteResult.row_count(r: &SqliteResult) -> Int {
  return r.rows.len();
}

fn SqliteResult.column_count(r: &SqliteResult) -> Int {
  return r.columns.len();
}

fn SqliteAffinity.to_str(a: SqliteAffinity) -> Str {
  match a {
    Text => { return "TEXT"; }
    Numeric => { return "NUMERIC"; }
    Integer => { return "INTEGER"; }
    Real => { return "REAL"; }
    Blob => { return "BLOB"; }
  }
}

fn table_def_new(name: Str) -> TableDef {
  var columns = Vec[ColumnDef].new();
  return TableDef{ name: name, columns: columns };
}

fn table_def_add_column(table: &mut TableDef, name: Str, affinity: SqliteAffinity, nullable: Bool, primary_key: Bool) {
  var col = ColumnDef{ name: name, affinity: affinity, nullable: nullable, primary_key: primary_key };
  table.columns.push(col);
}

fn table_def_column_count(table: &TableDef) -> Int {
  return table.columns.len();
}

fn table_def_to_create_sql(table: &TableDef) -> Str {
  var sql = "CREATE TABLE " + table.name + " (";
  var i = 0;
  while i < table.columns.len() {
    if i > 0 { sql = sql + ", "; }
    var col = table.columns[i];
    sql = sql + col.name + " " + SqliteAffinity.to_str(col.affinity);
    if col.primary_key { sql = sql + " PRIMARY KEY"; }
    if !col.nullable { sql = sql + " NOT NULL"; }
    i = i + 1;
  }
  sql = sql + ");";
  return sql;
}

// ============================================================================
// Tests
// ============================================================================

fn test_sqlite_row_new_empty() -> Bool {
  let row = SqliteRow.new();
  return SqliteRow.len(&row) == 0;
}

fn test_sqlite_row_add_get() -> Bool {
  var row = SqliteRow.new();
  SqliteRow.add_value(&mut row, SqliteValue.Integer(42));
  SqliteRow.add_value(&mut row, SqliteValue.Text("hello"));
  SqliteRow.add_value(&mut row, SqliteValue.Null);
  let v0 = SqliteRow.get_value(&row, 0);
  let v1 = SqliteRow.get_value(&row, 1);
  let v2 = SqliteRow.get_value(&row, 2);
  if v0.is_some() && v1.is_some() && v2.is_some() {
    let i0 = SqliteValue.as_int(&v0.unwrap());
    let t1 = SqliteValue.as_text(&v1.unwrap());
    let n2 = SqliteValue.is_null(&v2.unwrap());
    if i0.is_some() && t1.is_some() {
      return i0.unwrap() == 42 && t1.unwrap() == "hello" && n2;
    }
  }
  return false;
}

fn test_sqlite_row_get_out_of_bounds() -> Bool {
  var row = SqliteRow.new();
  SqliteRow.add_value(&mut row, SqliteValue.Integer(1));
  let v_neg = SqliteRow.get_value(&row, -1);
  let v_high = SqliteRow.get_value(&row, 5);
  return v_neg.is_none() && v_high.is_none();
}

fn test_sqlite_value_is_null() -> Bool {
  let v = SqliteValue.Null;
  return SqliteValue.is_null(&v) && !SqliteValue.is_integer(&v);
}

fn test_sqlite_value_is_integer() -> Bool {
  let v = SqliteValue.Integer(100);
  return SqliteValue.is_integer(&v) && !SqliteValue.is_text(&v);
}

fn test_sqlite_value_is_real() -> Bool {
  let v = SqliteValue.Real(3.14);
  return SqliteValue.is_real(&v) && !SqliteValue.is_blob(&v);
}

fn test_sqlite_value_is_text() -> Bool {
  let v = SqliteValue.Text("data");
  return SqliteValue.is_text(&v) && !SqliteValue.is_null(&v);
}

fn test_sqlite_value_is_blob() -> Bool {
  var blob = Vec[UInt8].new();
  blob.push(0xDE);
  blob.push(0xAD);
  let v = SqliteValue.Blob(blob);
  return SqliteValue.is_blob(&v) && !SqliteValue.is_real(&v);
}

fn test_sqlite_value_as_int_valid() -> Bool {
  let v = SqliteValue.Integer(-7);
  let opt = SqliteValue.as_int(&v);
  if opt.is_some() { return opt.unwrap() == -7; }
  return false;
}

fn test_sqlite_value_as_int_invalid() -> Bool {
  let v = SqliteValue.Text("not an int");
  return SqliteValue.as_int(&v).is_none();
}

fn test_sqlite_value_as_real_valid() -> Bool {
  let v = SqliteValue.Real(2.718);
  let opt = SqliteValue.as_real(&v);
  if opt.is_some() { return opt.unwrap() == 2.718; }
  return false;
}

fn test_sqlite_value_as_real_invalid() -> Bool {
  let v = SqliteValue.Null;
  return SqliteValue.as_real(&v).is_none();
}

fn test_sqlite_value_as_text_valid() -> Bool {
  let v = SqliteValue.Text("xiom");
  let opt = SqliteValue.as_text(&v);
  if opt.is_some() { return opt.unwrap() == "xiom"; }
  return false;
}

fn test_sqlite_value_as_text_invalid() -> Bool {
  let v = SqliteValue.Integer(10);
  return SqliteValue.as_text(&v).is_none();
}

fn test_sqlite_result_new_empty() -> Bool {
  let r = SqliteResult.new();
  return SqliteResult.row_count(&r) == 0 && SqliteResult.column_count(&r) == 0;
}

fn test_sqlite_result_add_column_and_row() -> Bool {
  var r = SqliteResult.new();
  SqliteResult.add_column(&mut r, "id");
  SqliteResult.add_column(&mut r, "name");
  var row = SqliteRow.new();
  SqliteRow.add_value(&mut row, SqliteValue.Integer(1));
  SqliteRow.add_value(&mut row, SqliteValue.Text("Alice"));
  SqliteResult.add_row(&mut r, row);
  return SqliteResult.column_count(&r) == 2 && SqliteResult.row_count(&r) == 1;
}

fn test_sqlite_row_multiple_types() -> Bool {
  var row = SqliteRow.new();
  SqliteRow.add_value(&mut row, SqliteValue.Null);
  SqliteRow.add_value(&mut row, SqliteValue.Integer(42));
  SqliteRow.add_value(&mut row, SqliteValue.Real(1.5));
  SqliteRow.add_value(&mut row, SqliteValue.Text("str"));
  if SqliteRow.len(&row) != 4 { return false; }
  return SqliteValue.is_null(&SqliteRow.get_value(&row, 0).unwrap())
    && SqliteValue.is_integer(&SqliteRow.get_value(&row, 1).unwrap())
    && SqliteValue.is_real(&SqliteRow.get_value(&row, 2).unwrap())
    && SqliteValue.is_text(&SqliteRow.get_value(&row, 3).unwrap());
}

fn test_table_def_new() -> Bool {
  let t = table_def_new("users");
  return t.name == "users" && table_def_column_count(&t) == 0;
}

fn test_table_def_add_column() -> Bool {
  var t = table_def_new("products");
  table_def_add_column(&mut t, "id", SqliteAffinity.Integer, false, true);
  table_def_add_column(&mut t, "name", SqliteAffinity.Text, false, false);
  table_def_add_column(&mut t, "price", SqliteAffinity.Real, true, false);
  return table_def_column_count(&t) == 3;
}

fn test_table_def_to_create_sql_simple() -> Bool {
  var t = table_def_new("users");
  table_def_add_column(&mut t, "id", SqliteAffinity.Integer, false, true);
  table_def_add_column(&mut t, "name", SqliteAffinity.Text, false, false);
  let sql = table_def_to_create_sql(&t);
  return sql == "CREATE TABLE users (id INTEGER PRIMARY KEY NOT NULL, name TEXT NOT NULL);";
}

fn test_table_def_to_create_sql_nullable() -> Bool {
  var t = table_def_new("items");
  table_def_add_column(&mut t, "id", SqliteAffinity.Integer, false, true);
  table_def_add_column(&mut t, "description", SqliteAffinity.Text, true, false);
  let sql = table_def_to_create_sql(&t);
  return sql == "CREATE TABLE items (id INTEGER PRIMARY KEY NOT NULL, description TEXT);";
}

fn test_table_def_to_create_sql_all_types() -> Bool {
  var t = table_def_new("inventory");
  table_def_add_column(&mut t, "id", SqliteAffinity.Integer, false, true);
  table_def_add_column(&mut t, "amount", SqliteAffinity.Numeric, false, false);
  table_def_add_column(&mut t, "price", SqliteAffinity.Real, false, false);
  table_def_add_column(&mut t, "data", SqliteAffinity.Blob, true, false);
  table_def_add_column(&mut t, "label", SqliteAffinity.Text, false, false);
  let sql = table_def_to_create_sql(&t);
  return sql == "CREATE TABLE inventory (id INTEGER PRIMARY KEY NOT NULL, amount NUMERIC NOT NULL, price REAL NOT NULL, data BLOB, label TEXT NOT NULL);";
}

fn test_sqlite_affinity_to_str() -> Bool {
  return SqliteAffinity.to_str(SqliteAffinity.Text) == "TEXT"
    && SqliteAffinity.to_str(SqliteAffinity.Integer) == "INTEGER"
    && SqliteAffinity.to_str(SqliteAffinity.Real) == "REAL"
    && SqliteAffinity.to_str(SqliteAffinity.Blob) == "BLOB"
    && SqliteAffinity.to_str(SqliteAffinity.Numeric) == "NUMERIC";
}

// ============================================================================
// Main
// ============================================================================

fn main() -> Int {
  var passed = 0;
  var total = 0;

  total = total + 1;
  if test_sqlite_row_new_empty() { passed = passed + 1; }

  total = total + 1;
  if test_sqlite_row_add_get() { passed = passed + 1; }

  total = total + 1;
  if test_sqlite_row_get_out_of_bounds() { passed = passed + 1; }

  total = total + 1;
  if test_sqlite_value_is_null() { passed = passed + 1; }

  total = total + 1;
  if test_sqlite_value_is_integer() { passed = passed + 1; }

  total = total + 1;
  if test_sqlite_value_is_real() { passed = passed + 1; }

  total = total + 1;
  if test_sqlite_value_is_text() { passed = passed + 1; }

  total = total + 1;
  if test_sqlite_value_is_blob() { passed = passed + 1; }

  total = total + 1;
  if test_sqlite_value_as_int_valid() { passed = passed + 1; }

  total = total + 1;
  if test_sqlite_value_as_int_invalid() { passed = passed + 1; }

  total = total + 1;
  if test_sqlite_value_as_real_valid() { passed = passed + 1; }

  total = total + 1;
  if test_sqlite_value_as_real_invalid() { passed = passed + 1; }

  total = total + 1;
  if test_sqlite_value_as_text_valid() { passed = passed + 1; }

  total = total + 1;
  if test_sqlite_value_as_text_invalid() { passed = passed + 1; }

  total = total + 1;
  if test_sqlite_result_new_empty() { passed = passed + 1; }

  total = total + 1;
  if test_sqlite_result_add_column_and_row() { passed = passed + 1; }

  total = total + 1;
  if test_sqlite_row_multiple_types() { passed = passed + 1; }

  total = total + 1;
  if test_table_def_new() { passed = passed + 1; }

  total = total + 1;
  if test_table_def_add_column() { passed = passed + 1; }

  total = total + 1;
  if test_table_def_to_create_sql_simple() { passed = passed + 1; }

  total = total + 1;
  if test_table_def_to_create_sql_nullable() { passed = passed + 1; }

  total = total + 1;
  if test_table_def_to_create_sql_all_types() { passed = passed + 1; }

  total = total + 1;
  if test_sqlite_affinity_to_str() { passed = passed + 1; }

  if passed == total { return 0; }
  return 1;
}
