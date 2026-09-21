// XIOM -- Collections Conformance Tests
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module collections_tests
use xiom.test;

// === Vec Tests ===
fn test_vec_new_empty() -> TestResult { let v = Vec[Int].new(); return assert(v.len() == 0, "Vec::new empty"); }
fn test_vec_push_one() -> TestResult { var v = Vec[Int].new(); v.push(42); if v.len()==1 { return assert(true, "Vec::push one"); } return assert(false, "Vec::push one"); }
fn test_vec_push_many() -> TestResult { var v = Vec[Int].new(); v.push(1); v.push(2); v.push(3); if v.len()==3 { return assert(true, "Vec::push many"); } return assert(false, "Vec::push many"); }
fn test_vec_pop_empty() -> TestResult { var v = Vec[Int].new(); if v.pop().is_none() { return assert(true, "Vec::pop empty"); } return assert(false, "Vec::pop empty"); }
fn test_vec_pop_nonempty() -> TestResult { var v = Vec[Int].new(); v.push(42); if v.pop().unwrap() == 42 { return assert(true, "Vec::pop nonempty"); } return assert(false, "Vec::pop nonempty"); }
fn test_vec_get_valid() -> TestResult { var v = Vec[Int].new(); v.push(10); if v.get(0).unwrap() == 10 { return assert(true, "Vec::get valid"); } return assert(false, "Vec::get valid"); }
fn test_vec_get_invalid() -> TestResult { let v = Vec[Int].new(); if v.get(0).is_none() { return assert(true, "Vec::get invalid"); } return assert(false, "Vec::get invalid"); }
fn test_vec_is_empty() -> TestResult { let v = Vec[Int].new(); if v.is_empty() { return assert(true, "Vec::is_empty"); } return assert(false, "Vec::is_empty"); }
fn test_vec_clear() -> TestResult { var v = Vec[Int].new(); v.push(1); v.clear(); if v.is_empty() { return assert(true, "Vec::clear"); } return assert(false, "Vec::clear"); }
fn test_vec_lifo() -> TestResult { var v = Vec[Int].new(); v.push(1); v.push(2); v.push(3); if v.pop()==Some(3)&&v.pop()==Some(2)&&v.pop()==Some(1) { return assert(true, "Vec LIFO"); } return assert(false, "Vec LIFO"); }
fn test_vec_insert() -> TestResult { var v = Vec[Int].new(); v.push(1); v.push(3); v.insert(1, 2); if v.get(0).unwrap()==1&&v.get(1).unwrap()==2&&v.get(2).unwrap()==3 { return assert(true, "Vec::insert"); } return assert(false, "Vec::insert"); }
fn test_vec_remove() -> TestResult { var v = Vec[Int].new(); v.push(1); v.push(2); v.push(3); let val = v.remove(1); if val==Some(2)&&v.len()==2&&v.get(0).unwrap()==1&&v.get(1).unwrap()==3 { return assert(true, "Vec::remove"); } return assert(false, "Vec::remove"); }
fn test_vec_remove_oob() -> TestResult { var v = Vec[Int].new(); if v.remove(0).is_none()&&v.remove(-1).is_none() { return assert(true, "Vec::remove OOB"); } return assert(false, "Vec::remove OOB"); }
fn test_vec_first() -> TestResult { var v = Vec[Int].new(); v.push(10); v.push(20); if v.first().unwrap()==10 { return assert(true, "Vec::first"); } return assert(false, "Vec::first"); }
fn test_vec_first_empty() -> TestResult { let v = Vec[Int].new(); if v.first().is_none() { return assert(true, "Vec::first empty"); } return assert(false, "Vec::first empty"); }
fn test_vec_last() -> TestResult { var v = Vec[Int].new(); v.push(10); v.push(20); if v.last().unwrap()==20 { return assert(true, "Vec::last"); } return assert(false, "Vec::last"); }
fn test_vec_last_empty() -> TestResult { let v = Vec[Int].new(); if v.last().is_none() { return assert(true, "Vec::last empty"); } return assert(false, "Vec::last empty"); }
fn test_vec_set() -> TestResult { var v = Vec[Int].new(); v.push(1); v.set(0, 42); if v.get(0).unwrap()==42 { return assert(true, "Vec::set"); } return assert(false, "Vec::set"); }
fn test_vec_with_capacity() -> TestResult { var v = Vec[Int].with_capacity(16); if v.len()==0&&v.is_empty() { return assert(true, "Vec::with_capacity"); } return assert(false, "Vec::with_capacity"); }
fn test_vec_push_pop_cycle() -> TestResult { var v = Vec[Int].new(); v.push(1); v.push(2); if v.pop()==Some(2) { v.push(3); v.push(4); if v.len()==3&&v.pop()==Some(4) { return assert(true, "Vec::push/pop cycle"); } } return assert(false, "Vec::push/pop cycle"); }

// === Map Tests ===
fn test_map_new_empty() -> TestResult { let m = Map[Str, Int].new(); return assert(m.len() == 0, "Map::new empty"); }
fn test_map_insert_get() -> TestResult { var m = Map[Str, Int].new(); m.insert("a", 1); if m.get("a").unwrap()==1 { return assert(true, "Map::insert/get"); } return assert(false, "Map::insert/get"); }
fn test_map_get_missing() -> TestResult { let m = Map[Str, Int].new(); if m.get("x").is_none() { return assert(true, "Map::get missing"); } return assert(false, "Map::get missing"); }
fn test_map_contains() -> TestResult { var m = Map[Str, Int].new(); m.insert("a", 1); if m.contains("a")&&!m.contains("b") { return assert(true, "Map::contains"); } return assert(false, "Map::contains"); }
fn test_map_remove() -> TestResult { var m = Map[Str, Int].new(); m.insert("a", 1); let v = m.remove("a"); if v==Some(1)&&!m.contains("a") { return assert(true, "Map::remove"); } return assert(false, "Map::remove"); }
fn test_map_keys() -> TestResult { var m = Map[Str, Int].new(); m.insert("a", 1); m.insert("b", 2); let keys = m.keys(); if keys.len()==2 { return assert(true, "Map::keys"); } return assert(false, "Map::keys"); }
fn test_map_values() -> TestResult { var m = Map[Str, Int].new(); m.insert("a", 1); m.insert("b", 2); let vals = m.values(); if vals.len()==2 { return assert(true, "Map::values"); } return assert(false, "Map::values"); }
fn test_map_overwrite() -> TestResult { var m = Map[Str, Int].new(); m.insert("a", 1); m.insert("a", 2); if m.get("a").unwrap()==2&&m.len()==1 { return assert(true, "Map::overwrite"); } return assert(false, "Map::overwrite"); }
fn test_map_remove_missing() -> TestResult { var m = Map[Str, Int].new(); if m.remove("x").is_none() { return assert(true, "Map::remove missing"); } return assert(false, "Map::remove missing"); }
fn test_map_clear() -> TestResult { var m = Map[Str, Int].new(); m.insert("a", 1); m.insert("b", 2); m.clear(); if m.len()==0 { return assert(true, "Map::clear"); } return assert(false, "Map::clear"); }

// === Set Tests ===
fn test_set_insert_contains() -> TestResult { var s = Set[Int].new(); s.insert(42); if s.contains(42)&&!s.contains(43) { return assert(true, "Set::insert/contains"); } return assert(false, "Set::insert/contains"); }
fn test_set_remove() -> TestResult { var s = Set[Int].new(); s.insert(42); s.remove(42); if !s.contains(42) { return assert(true, "Set::remove"); } return assert(false, "Set::remove"); }
fn test_set_empty() -> TestResult { let s = Set[Int].new(); if s.is_empty() { return assert(true, "Set empty"); } return assert(false, "Set empty"); }
fn test_set_len() -> TestResult { var s = Set[Int].new(); s.insert(1); s.insert(2); if s.len()==2 { return assert(true, "Set::len"); } return assert(false, "Set::len"); }
fn test_set_no_duplicates() -> TestResult { var s = Set[Int].new(); s.insert(42); s.insert(42); if s.len()==1 { return assert(true, "Set::no duplicates"); } return assert(false, "Set::no duplicates"); }
fn test_set_union() -> TestResult { var s1 = Set[Int].new(); s1.insert(1); s1.insert(2); var s2 = Set[Int].new(); s2.insert(2); s2.insert(3); let u = s1.union(s2); if u.contains(1)&&u.contains(2)&&u.contains(3)&&u.len()==3 { return assert(true, "Set::union"); } return assert(false, "Set::union"); }
fn test_set_intersection() -> TestResult { var s1 = Set[Int].new(); s1.insert(1); s1.insert(2); var s2 = Set[Int].new(); s2.insert(2); s2.insert(3); let i = s1.intersection(s2); if i.contains(2)&&i.len()==1 { return assert(true, "Set::intersection"); } return assert(false, "Set::intersection"); }
fn test_set_difference() -> TestResult { var s1 = Set[Int].new(); s1.insert(1); s1.insert(2); s1.insert(3); var s2 = Set[Int].new(); s2.insert(2); let d = s1.difference(s2); if d.contains(1)&&d.contains(3)&&!d.contains(2)&&d.len()==2 { return assert(true, "Set::difference"); } return assert(false, "Set::difference"); }

// === Stack Tests ===
fn test_stack_push_pop() -> TestResult { var s = Stack[Int].new(); s.push(1); s.push(2); s.push(3); if s.pop()==Some(3)&&s.pop()==Some(2)&&s.pop()==Some(1) { return assert(true, "Stack::push/pop LIFO"); } return assert(false, "Stack::push/pop LIFO"); }
fn test_stack_peek() -> TestResult { var s = Stack[Int].new(); s.push(42); if s.peek()==Some(42)&&s.len()==1 { return assert(true, "Stack::peek"); } return assert(false, "Stack::peek"); }
fn test_stack_pop_empty() -> TestResult { var s = Stack[Int].new(); if s.pop().is_none() { return assert(true, "Stack::pop empty"); } return assert(false, "Stack::pop empty"); }
fn test_stack_is_empty() -> TestResult { var s = Stack[Int].new(); if s.is_empty() { return assert(true, "Stack::is_empty"); } return assert(false, "Stack::is_empty"); }

// === Queue Tests ===
fn test_queue_enqueue_dequeue() -> TestResult { var q = Queue[Int].new(); q.enqueue(1); q.enqueue(2); q.enqueue(3); if q.dequeue()==Some(1)&&q.dequeue()==Some(2)&&q.dequeue()==Some(3) { return assert(true, "Queue::FIFO"); } return assert(false, "Queue::FIFO"); }
fn test_queue_peek() -> TestResult { var q = Queue[Int].new(); q.enqueue(42); if q.peek()==Some(42)&&q.len()==1 { return assert(true, "Queue::peek"); } return assert(false, "Queue::peek"); }
fn test_queue_dequeue_empty() -> TestResult { var q = Queue[Int].new(); if q.dequeue().is_none() { return assert(true, "Queue::dequeue empty"); } return assert(false, "Queue::dequeue empty"); }
fn test_queue_is_empty() -> TestResult { var q = Queue[Int].new(); if q.is_empty() { return assert(true, "Queue::is_empty"); } return assert(false, "Queue::is_empty"); }

// === LinkedList Tests ===
fn test_linkedlist_push_front_pop_front() -> TestResult { var ll = LinkedList[Int].new(); ll.push_front(1); ll.push_front(2); if ll.pop_front()==Some(2)&&ll.pop_front()==Some(1) { return assert(true, "LinkedList::push_front/pop_front"); } return assert(false, "LinkedList::push_front/pop_front"); }
fn test_linkedlist_push_back_pop_back() -> TestResult { var ll = LinkedList[Int].new(); ll.push_back(1); ll.push_back(2); if ll.pop_back()==Some(2)&&ll.pop_back()==Some(1) { return assert(true, "LinkedList::push_back/pop_back"); } return assert(false, "LinkedList::push_back/pop_back"); }
fn test_linkedlist_mixed() -> TestResult { var ll = LinkedList[Int].new(); ll.push_back(1); ll.push_front(0); ll.push_back(2); if ll.pop_front()==Some(0)&&ll.pop_back()==Some(2)&&ll.pop_front()==Some(1) { return assert(true, "LinkedList::mixed"); } return assert(false, "LinkedList::mixed"); }
fn test_linkedlist_pop_empty() -> TestResult { var ll = LinkedList[Int].new(); if ll.pop_front().is_none()&&ll.pop_back().is_none() { return assert(true, "LinkedList::pop empty"); } return assert(false, "LinkedList::pop empty"); }

// === VecDeque Tests ===
fn test_vecdeque_push_back_pop_front() -> TestResult { var d = VecDeque[Int].new(); d.push_back(1); d.push_back(2); if d.pop_front()==Some(1)&&d.pop_front()==Some(2) { return assert(true, "VecDeque::push_back/pop_front"); } return assert(false, "VecDeque::push_back/pop_front"); }
fn test_vecdeque_push_front_pop_back() -> TestResult { var d = VecDeque[Int].new(); d.push_front(1); d.push_front(2); if d.pop_back()==Some(1)&&d.pop_back()==Some(2) { return assert(true, "VecDeque::push_front/pop_back"); } return assert(false, "VecDeque::push_front/pop_back"); }
fn test_vecdeque_mixed() -> TestResult { var d = VecDeque[Int].new(); d.push_back(2); d.push_front(1); d.push_back(3); if d.pop_front()==Some(1)&&d.pop_back()==Some(3)&&d.pop_front()==Some(2) { return assert(true, "VecDeque::mixed"); } return assert(false, "VecDeque::mixed"); }
fn test_vecdeque_len() -> TestResult { var d = VecDeque[Int].new(); d.push_back(1); d.push_front(0); if d.len()==2 { return assert(true, "VecDeque::len"); } return assert(false, "VecDeque::len"); }

// === BTreeMap Tests ===
fn test_btreemap_insert_get() -> TestResult { var m = BTreeMap[Int, Str].new(); m.insert(3, "c"); m.insert(1, "a"); m.insert(2, "b"); if m.get(1).unwrap()=="a"&&m.get(2).unwrap()=="b"&&m.get(3).unwrap()=="c" { return assert(true, "BTreeMap::insert/get sorted"); } return assert(false, "BTreeMap::insert/get sorted"); }
fn test_btreemap_first_last() -> TestResult { var m = BTreeMap[Int, Str].new(); m.insert(3, "c"); m.insert(1, "a"); let first = m.first_entry(); let last = m.last_entry(); if first.is_some()&&last.is_some() { return assert(true, "BTreeMap::first/last entry"); } return assert(false, "BTreeMap::first/last entry"); }
fn test_btreemap_remove() -> TestResult { var m = BTreeMap[Int, Str].new(); m.insert(1, "a"); m.insert(2, "b"); let v = m.remove(1); if v==Some("a")&&!m.contains_key(1)&&m.len()==1 { return assert(true, "BTreeMap::remove"); } return assert(false, "BTreeMap::remove"); }
fn test_btreemap_contains_key() -> TestResult { var m = BTreeMap[Int, Str].new(); m.insert(1, "a"); if m.contains_key(1)&&!m.contains_key(2) { return assert(true, "BTreeMap::contains_key"); } return assert(false, "BTreeMap::contains_key"); }

// === BTreeSet Tests ===
fn test_btreeset_insert_contains() -> TestResult { var s = BTreeSet[Int].new(); s.insert(3); s.insert(1); s.insert(2); if s.contains(1)&&s.contains(2)&&s.contains(3) { return assert(true, "BTreeSet::insert/contains sorted"); } return assert(false, "BTreeSet::insert/contains sorted"); }
fn test_btreeset_first_last() -> TestResult { var s = BTreeSet[Int].new(); s.insert(3); s.insert(1); if s.first()==Some(1)&&s.last()==Some(3) { return assert(true, "BTreeSet::first/last"); } return assert(false, "BTreeSet::first/last"); }
fn test_btreeset_no_duplicates() -> TestResult { var s = BTreeSet[Int].new(); s.insert(42); s.insert(42); if s.len()==1 { return assert(true, "BTreeSet::no duplicates"); } return assert(false, "BTreeSet::no duplicates"); }

// === Slice Tests ===
fn test_slice_len_empty() -> TestResult { var s = Slice[Int]{ data: Vec[Int].new() }; if s.len()==0&&s.is_empty() { return assert(true, "Slice::len/empty"); } return assert(false, "Slice::len/empty"); }
fn test_slice_first_last() -> TestResult { var s = Slice[Int]{ data: Vec[Int].new() }; s.data.push(1); s.data.push(2); if s.first()==Some(1)&&s.last()==Some(2) { return assert(true, "Slice::first/last"); } return assert(false, "Slice::first/last"); }
fn test_slice_get() -> TestResult { var s = Slice[Int]{ data: Vec[Int].new() }; s.data.push(10); s.data.push(20); if s.get(0)==Some(10)&&s.get(1)==Some(20)&&s.get(2).is_none()&&s.get(-1).is_none() { return assert(true, "Slice::get"); } return assert(false, "Slice::get"); }

fn main() -> Int {
  var tests = [
    test_vec_new_empty, test_vec_push_one, test_vec_push_many, test_vec_pop_empty, test_vec_pop_nonempty,
    test_vec_get_valid, test_vec_get_invalid, test_vec_is_empty, test_vec_clear, test_vec_lifo,
    test_vec_insert, test_vec_remove, test_vec_remove_oob, test_vec_first, test_vec_first_empty,
    test_vec_last, test_vec_last_empty, test_vec_set, test_vec_with_capacity, test_vec_push_pop_cycle,
    test_map_new_empty, test_map_insert_get, test_map_get_missing, test_map_contains, test_map_remove,
    test_map_keys, test_map_values, test_map_overwrite, test_map_remove_missing, test_map_clear,
    test_set_insert_contains, test_set_remove, test_set_empty, test_set_len, test_set_no_duplicates,
    test_set_union, test_set_intersection, test_set_difference,
    test_stack_push_pop, test_stack_peek, test_stack_pop_empty, test_stack_is_empty,
    test_queue_enqueue_dequeue, test_queue_peek, test_queue_dequeue_empty, test_queue_is_empty,
    test_linkedlist_push_front_pop_front, test_linkedlist_push_back_pop_back, test_linkedlist_mixed, test_linkedlist_pop_empty,
    test_vecdeque_push_back_pop_front, test_vecdeque_push_front_pop_back, test_vecdeque_mixed, test_vecdeque_len,
    test_btreemap_insert_get, test_btreemap_first_last, test_btreemap_remove, test_btreemap_contains_key,
    test_btreeset_insert_contains, test_btreeset_first_last, test_btreeset_no_duplicates,
    test_slice_len_empty, test_slice_first_last, test_slice_get
  ];
  return test.run_all(tests);
}
