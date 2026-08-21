// XIOM -- Memory Module Conformance Tests
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

module memory_tests
use xiom.test;
use xiom.mem;
use xiom.ptr;
use xiom.alloc;
use xiom.array;
use xiom.cell;
use xiom.rc;
use xiom.ffi;

// ============================================================
// Helper functions for array map/fold tests
// ============================================================

fn map_inc(x: Int) -> Int { return x + 1; }
fn fold_sum(acc: Int, x: Int) -> Int { return acc + x; }
fn fold_count(acc: Int, x: Int) -> Int { return acc + 1; }

// ============================================================
// xiom.mem tests
// ============================================================

fn test_mem_manually_drop_new() -> TestResult {
  let md = mem.ManuallyDrop.new(42);
  if md.into_inner() == 42 { return assert(true, "mem::ManuallyDrop.new + into_inner"); }
  return assert(false, "mem::ManuallyDrop.new + into_inner");
}

fn test_mem_size_of_int() -> TestResult {
  if mem.size_of[Int]() > 0 { return assert(true, "mem::size_of[Int] > 0"); }
  return assert(false, "mem::size_of[Int] > 0");
}

fn test_mem_size_of_bool() -> TestResult {
  if mem.size_of[Bool]() > 0 { return assert(true, "mem::size_of[Bool] > 0"); }
  return assert(false, "mem::size_of[Bool] > 0");
}

fn test_mem_swap_basic() -> TestResult {
  var a = 10;
  var b = 20;
  mem.swap(&mut a, &mut b);
  if a == 20 && b == 10 { return assert(true, "mem::swap basic"); }
  return assert(false, "mem::swap basic");
}

fn test_mem_replace_basic() -> TestResult {
  var x = 10;
  let old = mem.replace(&mut x, 99);
  if old == 10 && x == 99 { return assert(true, "mem::replace basic"); }
  return assert(false, "mem::replace basic");
}

fn test_mem_take_with_default() -> TestResult {
  var x = 42;
  let old = mem.take(&mut x);
  if old == 42 && x == 0 { return assert(true, "mem::take with Default"); }
  return assert(false, "mem::take with Default");
}

// ============================================================
// xiom.ptr tests
// ============================================================

fn test_ptr_null_is_null() -> TestResult {
  let p = ptr.null[Int]();
  if ptr.is_null(p) { return assert(true, "ptr::null + is_null"); }
  return assert(false, "ptr::null + is_null");
}

fn test_ptr_null_mut_is_null() -> TestResult {
  let p = ptr.null_mut[Int]();
  if ptr.is_null(p) { return assert(true, "ptr::null_mut + is_null"); }
  return assert(false, "ptr::null_mut + is_null");
}

fn test_ptr_dangling_not_null() -> TestResult {
  let p = ptr.dangling[Int]();
  if !ptr.is_null(p) { return assert(true, "ptr::dangling not null"); }
  return assert(false, "ptr::dangling not null");
}

fn test_ptr_from_ref_roundtrip() -> TestResult {
  let x = 42;
  let p = ptr.from_ref(&x);
  unsafe {
    let v = ptr.read(p);
    if v == 42 { return assert(true, "ptr::from_ref roundtrip"); }
  }
  return assert(false, "ptr::from_ref roundtrip");
}

fn test_ptr_write_read_unsafe() -> TestResult {
  var x = 0;
  let p = ptr.from_mut(&mut x);
  unsafe {
    ptr.write(p, 99);
    let v = ptr.read(p);
    if v == 99 { return assert(true, "ptr::write + read unsafe"); }
  }
  return assert(false, "ptr::write + read unsafe");
}

fn test_ptr_swap_via_raw() -> TestResult {
  var a = 1;
  var b = 2;
  let pa = ptr.from_mut(&mut a);
  let pb = ptr.from_mut(&mut b);
  unsafe {
    ptr.swap(pa, pb);
  }
  if a == 2 && b == 1 { return assert(true, "ptr::swap via raw"); }
  return assert(false, "ptr::swap via raw");
}

fn test_ptr_replace_via_raw() -> TestResult {
  var x = 10;
  let p = ptr.from_mut(&mut x);
  unsafe {
    let old = ptr.replace(p, 55);
    if old == 10 && x == 55 { return assert(true, "ptr::replace via raw"); }
  }
  return assert(false, "ptr::replace via raw");
}

// ============================================================
// xiom.alloc tests
// ============================================================

fn test_alloc_non_null() -> TestResult {
  let p = alloc.alloc(64);
  if !ptr.is_null(p) { return assert(true, "alloc::alloc non-null"); }
  return assert(false, "alloc::alloc non-null");
}

fn test_alloc_dealloc_roundtrip() -> TestResult {
  let p = alloc.alloc(128);
  if ptr.is_null(p) { return assert(false, "alloc::alloc returned null"); }
  alloc.dealloc(p, 128);
  return assert(true, "alloc::alloc + dealloc roundtrip");
}

fn test_layout_new_size_align() -> TestResult {
  let layout = alloc.Layout.new(64);
  if layout.size == 64 && layout.align == 8 { return assert(true, "alloc::Layout.new size/align"); }
  return assert(false, "alloc::Layout.new size/align");
}

fn test_layout_padded_size() -> TestResult {
  let layout = alloc.Layout.new(13);
  let padded = layout.padded_size();
  if padded >= 13 && padded % 8 == 0 { return assert(true, "alloc::Layout.padded_size"); }
  return assert(false, "alloc::Layout.padded_size");
}

fn test_alloc_zeroed_non_null() -> TestResult {
  let p = alloc.alloc_zeroed(32);
  if !ptr.is_null(p) { return assert(true, "alloc::alloc_zeroed non-null"); }
  return assert(false, "alloc::alloc_zeroed non-null");
}

fn test_alloc_layout_non_null() -> TestResult {
  let layout = alloc.Layout.new(48);
  let p = alloc.alloc_layout(layout);
  if !ptr.is_null(p) { return assert(true, "alloc::alloc_layout non-null"); }
  return assert(false, "alloc::alloc_layout non-null");
}

// ============================================================
// xiom.array tests (fixed-size array operations)
// ============================================================

fn test_array_len() -> TestResult {
  let arr = [1, 2, 3, 4, 5];
  if array.len(&arr) == 5 { return assert(true, "array::len correct"); }
  return assert(false, "array::len correct");
}

fn test_array_get_within_bounds() -> TestResult {
  let arr = [10, 20, 30];
  let item = array.get(&arr, 1);
  if item.is_some() == true { return assert(true, "array::get within bounds"); }
  return assert(false, "array::get within bounds");
}

fn test_array_get_out_of_bounds() -> TestResult {
  let arr = [10, 20, 30];
  let item = array.get(&arr, 5);
  if item.is_some() == false { return assert(true, "array::get out of bounds"); }
  return assert(false, "array::get out of bounds");
}

fn test_array_fill() -> TestResult {
  var arr = [0, 0, 0, 0];
  array.fill(&mut arr, 7);
  if arr[0] == 7 && arr[1] == 7 && arr[2] == 7 && arr[3] == 7 {
    return assert(true, "array::fill");
  }
  return assert(false, "array::fill");
}

fn test_array_map() -> TestResult {
  let arr = [1, 2, 3, 4];
  let mapped = array.map(arr, map_inc);
  if mapped[0] == 2 && mapped[1] == 3 && mapped[2] == 4 && mapped[3] == 5 {
    return assert(true, "array::map");
  }
  return assert(false, "array::map");
}

fn test_array_fold() -> TestResult {
  let arr = [1, 2, 3, 4, 5];
  let sum = array.fold(arr, 0, fold_sum);
  if sum == 15 { return assert(true, "array::fold sum"); }
  return assert(false, "array::fold sum");
}

fn test_array_contains_found() -> TestResult {
  let arr = [10, 20, 30, 40, 50];
  if array.contains(&arr, &30) { return assert(true, "array::contains found"); }
  return assert(false, "array::contains found");
}

fn test_array_contains_not_found() -> TestResult {
  let arr = [10, 20, 30, 40, 50];
  if !array.contains(&arr, &99) { return assert(true, "array::contains not found"); }
  return assert(false, "array::contains not found");
}

fn test_array_binary_search_found() -> TestResult {
  let arr = [1, 2, 3, 4, 5];
  let result = array.binary_search(&arr, &3);
  if result.is_ok() { return assert(true, "array::binary_search found"); }
  return assert(false, "array::binary_search found");
}

fn test_array_binary_search_not_found() -> TestResult {
  let arr = [1, 2, 3, 4, 5];
  let result = array.binary_search(&arr, &99);
  if !result.is_ok() { return assert(true, "array::binary_search not found"); }
  return assert(false, "array::binary_search not found");
}

fn test_array_is_empty_true() -> TestResult {
  let arr: [0]Int;
  if array.is_empty(&arr) { return assert(true, "array::is_empty true"); }
  return assert(false, "array::is_empty true");
}

fn test_array_first_and_last() -> TestResult {
  let arr = [10, 20, 30];
  let f = array.first(&arr);
  let l = array.last(&arr);
  if f.is_some() && l.is_some() { return assert(true, "array::first + last"); }
  return assert(false, "array::first + last");
}

// ============================================================
// xiom.cell tests
// ============================================================

fn test_cell_new_and_get() -> TestResult {
  let c = cell.Cell.new(42);
  if c.get() == 42 { return assert(true, "Cell::new + get"); }
  return assert(false, "Cell::new + get");
}

fn test_cell_set() -> TestResult {
  let c = cell.Cell.new(10);
  c.set(99);
  if c.get() == 99 { return assert(true, "Cell::set"); }
  return assert(false, "Cell::set");
}

fn test_cell_replace_returns_old() -> TestResult {
  let c = cell.Cell.new(10);
  let old = c.replace(55);
  if old == 10 && c.get() == 55 { return assert(true, "Cell::replace returns old"); }
  return assert(false, "Cell::replace returns old");
}

fn test_cell_swap() -> TestResult {
  let c1 = cell.Cell.new(1);
  let c2 = cell.Cell.new(2);
  c1.swap(&c2);
  if c1.get() == 2 && c2.get() == 1 { return assert(true, "Cell::swap"); }
  return assert(false, "Cell::swap");
}

fn test_refcell_borrow() -> TestResult {
  let rc = cell.RefCell.new(42);
  let r = rc.borrow();
  if r.get() == 42 { return assert(true, "RefCell::borrow"); }
  return assert(false, "RefCell::borrow");
}

fn test_refcell_borrow_mut() -> TestResult {
  let rc = cell.RefCell.new(10);
  let r = rc.borrow_mut();
  r.set(88);
  if r.get() == 88 { return assert(true, "RefCell::borrow_mut"); }
  return assert(false, "RefCell::borrow_mut");
}

fn test_refcell_try_borrow() -> TestResult {
  let rc = cell.RefCell.new(42);
  let r = rc.try_borrow();
  if r.is_some() { return assert(true, "RefCell::try_borrow"); }
  return assert(false, "RefCell::try_borrow");
}

fn test_refcell_replace() -> TestResult {
  let rc = cell.RefCell.new(10);
  let old = rc.replace(77);
  if old == 10 { return assert(true, "RefCell::replace"); }
  return assert(false, "RefCell::replace");
}

// ============================================================
// xiom.rc tests
// ============================================================

fn test_rc_new_and_get() -> TestResult {
  let rc1 = rc.Rc.new(42);
  if rc1.get() == 42 { return assert(true, "Rc::new + get"); }
  return assert(false, "Rc::new + get");
}

fn test_rc_clone_increases_strong_count() -> TestResult {
  let rc1 = rc.Rc.new(100);
  if rc1.strong_count() != 1 { return assert(false, "Rc::clone count initial"); }
  let rc2 = rc1.clone();
  if rc2.strong_count() == 2 { return assert(true, "Rc::clone increases strong_count"); }
  return assert(false, "Rc::clone increases strong_count");
}

fn test_rc_weak_downgrade_upgrade() -> TestResult {
  let rc1 = rc.Rc.new(77);
  let weak = rc1.downgrade();
  if weak.strong_count() == 1 { return assert(false, "Weak::strong_count expected 1"); }
  let upgraded = weak.upgrade();
  if upgraded.is_some() { return assert(true, "Rc::downgrade + Weak::upgrade"); }
  return assert(false, "Rc::downgrade + Weak::upgrade");
}

fn test_rc_multiple_clones_count() -> TestResult {
  let rc1 = rc.Rc.new(1);
  let rc2 = rc1.clone();
  let rc3 = rc1.clone();
  let rc4 = rc2.clone();
  if rc1.strong_count() == 4 { return assert(true, "Rc multiple clones strong_count"); }
  return assert(false, "Rc multiple clones strong_count");
}

fn test_rc_get_after_clone() -> TestResult {
  let rc1 = rc.Rc.new(42);
  let rc2 = rc1.clone();
  if rc2.get() == 42 { return assert(true, "Rc::get after clone"); }
  return assert(false, "Rc::get after clone");
}

fn test_rc_weak_count() -> TestResult {
  let rc1 = rc.Rc.new(10);
  let weak1 = rc1.downgrade();
  if weak1.weak_count() == 1 { return assert(false, "Weak::weak_count expected >=1"); }
  let weak2 = rc1.downgrade();
  if weak1.weak_count() >= 1 { return assert(true, "Rc::downgrade weak_count"); }
  return assert(false, "Rc::downgrade weak_count");
}

// ============================================================
// xiom.ffi tests
// ============================================================

fn test_ffi_alloc_free_roundtrip() -> TestResult {
  let buf = ffi.alloc(128);
  if ptr.is_null(buf) { return assert(false, "ffi::alloc returned null"); }
  ffi.free(buf);
  return assert(true, "ffi::alloc + free roundtrip");
}

fn test_ffi_alloc_memcpy_free() -> TestResult {
  let buf = ffi.alloc(32);
  if ptr.is_null(buf) { return assert(false, "ffi::alloc returned null"); }
  ffi.memcpy(buf, buf, 16);
  ffi.free(buf);
  return assert(true, "ffi::alloc + memcpy + free roundtrip");
}

fn test_ffi_cstr_basic() -> TestResult {
  let buf = ffi.alloc(5);
  if ptr.is_null(buf) { return assert(false, "ffi::alloc for cstr returned null"); }
  unsafe {
    ptr.write(buf, 72); // 'H'
    ptr.write(buf + 1, 105); // 'i'
    ptr.write(buf + 2, 0); // null terminator
  }
  ffi.free(buf);
  return assert(true, "ffi::cstr basic write + free");
}

// ============================================================
// Test runner
// ============================================================

fn main() -> Int {
  var tests: Vec[fn() -> TestResult] = [
    // mem
    test_mem_manually_drop_new,
    test_mem_size_of_int,
    test_mem_size_of_bool,
    test_mem_swap_basic,
    test_mem_replace_basic,
    test_mem_take_with_default,
    // ptr
    test_ptr_null_is_null,
    test_ptr_null_mut_is_null,
    test_ptr_dangling_not_null,
    test_ptr_from_ref_roundtrip,
    test_ptr_write_read_unsafe,
    test_ptr_swap_via_raw,
    test_ptr_replace_via_raw,
    // alloc
    test_alloc_non_null,
    test_alloc_dealloc_roundtrip,
    test_layout_new_size_align,
    test_layout_padded_size,
    test_alloc_zeroed_non_null,
    test_alloc_layout_non_null,
    // array
    test_array_len,
    test_array_get_within_bounds,
    test_array_get_out_of_bounds,
    test_array_fill,
    test_array_map,
    test_array_fold,
    test_array_contains_found,
    test_array_contains_not_found,
    test_array_binary_search_found,
    test_array_binary_search_not_found,
    test_array_is_empty_true,
    test_array_first_and_last,
    // cell
    test_cell_new_and_get,
    test_cell_set,
    test_cell_replace_returns_old,
    test_cell_swap,
    test_refcell_borrow,
    test_refcell_borrow_mut,
    test_refcell_try_borrow,
    test_refcell_replace,
    // rc
    test_rc_new_and_get,
    test_rc_clone_increases_strong_count,
    test_rc_weak_downgrade_upgrade,
    test_rc_multiple_clones_count,
    test_rc_get_after_clone,
    test_rc_weak_count,
    // ffi
    test_ffi_alloc_free_roundtrip,
    test_ffi_alloc_memcpy_free,
    test_ffi_cstr_basic,
  ];
  return test.run_all(tests);
}
