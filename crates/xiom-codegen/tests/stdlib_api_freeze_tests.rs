// XIOM - Stdlib API Freeze Tests
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// FREEZE CONTRACT: every pub fn signature listed below is a public API
// guarantee. The stdlib may only ADD functions - renaming, removing, or
// changing the signature of a frozen function fails CI immediately.
//
// This is the mechanical enforcement of "don't break the ecosystem":
// 2,090 regression files, 687 smokes, 66 packages, and the E2E suites all
// depend on these signatures. Regenerate the snapshot ONLY as part of an
// intentional, reviewed API change.

use std::process::Command;
use std::path::Path;
use std::fs;

fn xiom_path() -> String {
    let mut path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap().parent().unwrap()
        .join("target").join("debug").join("xiom.exe");
    if !path.exists() {
        path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent().unwrap().parent().unwrap()
            .join("target").join("release").join("xiom.exe");
    }
    path.to_str().unwrap().to_string()
}

fn project_root() -> &'static Path {
    static ROOT: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();
    ROOT.get_or_init(|| {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent().unwrap().parent().unwrap()
            .to_path_buf()
    }).as_path()
}

/// Frozen snapshot: module :: pub fn signature for every public function
/// in the contract stdlib modules (post Tier-1 consolidation, 2026-08-07).
const FROZEN: &[&str] = &[
        "aes :: pub fn aes_sbox(b: Int) -> Int",
        "aes :: pub fn aes_inv_sbox(b: Int) -> Int",
        "aes :: pub fn aes_rcon(round: Int) -> Int",
        "aes :: pub fn aes128_encrypt(plaintext: &Vec[Int], key: &Vec[Int]) -> Result[Vec[Int], Str]",
        "aes :: pub fn aes128_decrypt(ciphertext: &Vec[Int], key: &Vec[Int]) -> Result[Vec[Int], Str]",
        "aes :: pub fn aes256_encrypt(plaintext: &Vec[Int], key: &Vec[Int]) -> Result[Vec[Int], Str]",
        "aes :: pub fn aes256_decrypt(ciphertext: &Vec[Int], key: &Vec[Int]) -> Result[Vec[Int], Str]",
        "alloc :: pub fn Layout.new(size: Int) -> Layout",
        "alloc :: pub fn Layout.with_align(self, align: Int) -> Layout",
        "alloc :: pub fn Layout.padded_size(self) -> Int",
        "alloc :: pub fn global_alloc() -> GlobalAlloc",
        "alloc :: pub fn GlobalAlloc.allocate(self, layout: Layout) -> Result<*mut UInt8, AllocError>",
        "alloc :: pub fn GlobalAlloc.deallocate(self, ptr: *mut UInt8, layout: Layout)",
        "alloc :: pub fn GlobalAlloc.allocate_zeroed(self, layout: Layout) -> Result<*mut UInt8, AllocError>",
        "alloc :: pub fn GlobalAlloc.grow(self, ptr: *mut UInt8, old: Layout, new: Layout) -> Result<*mut UInt8, AllocError>",
        "alloc :: pub fn GlobalAlloc.shrink(self, ptr: *mut UInt8, old: Layout, new: Layout) -> Result<*mut UInt8, AllocError>",
        "alloc :: pub fn alloc(size: Int) -> *mut UInt8",
        "alloc :: pub fn alloc_zeroed(size: Int) -> *mut UInt8",
        "alloc :: pub fn realloc_sized(ptr: *mut UInt8, old_size: Int, new_size: Int) -> *mut UInt8",
        "alloc :: pub fn dealloc(ptr: *mut UInt8, size: Int)",
        "alloc :: pub fn alloc_layout(layout: Layout) -> *mut UInt8",
        "alloc :: pub fn dealloc_layout(ptr: *mut UInt8, layout: Layout)",
        "array :: pub fn len[T, const N: Int](arr: &[N]T) -> Int",
        "array :: pub fn is_empty[T, const N: Int](arr: &[N]T) -> Bool",
        "array :: pub fn first[T, const N: Int](arr: &[N]T) -> Option[T]",
        "array :: pub fn last[T, const N: Int](arr: &[N]T) -> Option[T]",
        "array :: pub fn get[T, const N: Int](arr: &[N]T, index: Int) -> Option[T]",
        "array :: pub fn get_mut[T, const N: Int](arr: &mut [N]T, index: Int) -> Option[T]",
        "array :: pub fn map[T, U, const N: Int](arr: [N]T, f: fn(T) -> U) -> [N]U",
        "array :: pub fn zip[T, U, const N: Int](a: [N]T, b: [N]U) -> [N](T, U)",
        "array :: pub fn fold[T, B, const N: Int](arr: [N]T, init: B, f: fn(B, T) -> B) -> B",
        "array :: pub fn as_slice[T, const N: Int](arr: &[N]T) -> Slice[T]",
        "array :: pub fn as_mut_slice[T, const N: Int](arr: &mut [N]T) -> Slice[T]",
        "array :: pub fn each_ref[T, const N: Int](arr: &[N]T) -> [N]&T",
        "array :: pub fn each_mut[T, const N: Int](arr: &mut [N]T) -> [N]&mut T",
        "array :: pub fn fill[T: Clone, const N: Int](arr: &mut [N]T, value: T)",
        "array :: pub fn swap[T, const N: Int](arr: &mut [N]T, a: Int, b: Int)",
        "array :: pub fn reverse[T, const N: Int](arr: &mut [N]T)",
        "array :: pub fn rotate_left[T, const N: Int](arr: &mut [N]T, mid: Int)",
        "array :: pub fn rotate_right[T, const N: Int](arr: &mut [N]T, k: Int)",
        "array :: pub fn sort[T: Ord, const N: Int](arr: &mut [N]T)",
        "array :: pub fn sort_by[T, const N: Int](arr: &mut [N]T, compare: fn(&T, &T) -> Ordering)",
        "array :: pub fn binary_search[T: Ord, const N: Int](arr: &[N]T, x: &T) -> Result[Int, Int]",
        "array :: pub fn contains[T: Eq, const N: Int](arr: &[N]T, x: &T) -> Bool",
        "async :: pub fn Executor.new() -> Executor",
        "async :: pub fn Executor.spawn(self, task: fn())",
        "async :: pub fn Executor.at(self, deadline: Int, task: fn())",
        "async :: pub fn Executor.step(self) -> Bool",
        "async :: pub fn Executor.fire_due_timers(self)",
        "async :: pub fn Executor.run(self)",
        "async :: pub fn Executor.block_on(self, task: fn())",
        "async :: pub fn spawn(task: fn())",
        "async :: pub fn run()",
        "async :: pub fn block_on(task: fn())",
        "async :: pub fn delay(ms: Int, task: fn())",
        "async :: pub fn sleep_ms(ms: Int)",
        "async :: pub fn Channel.bounded[T](capacity: Int) -> Channel[T]",
        "async :: pub fn Channel.unbounded[T]() -> Channel[T]",
        "async :: pub fn Channel.send[T](&mut self, value: T)",
        "async :: pub fn Channel.recv[T](&mut self) -> T",
        "async :: pub fn Channel.try_recv[T](&mut self) -> Option[T]",
        "async :: pub fn Channel.close[T](self)",
        "bench :: pub fn run_bench(name: Str, f: fn()) -> BenchResult",
        "bench :: pub fn run_bench_n(name: Str, iterations: Int, f: fn()) -> BenchResult",
        "bench :: pub fn compare(a: BenchResult, b: BenchResult) -> Str",
        "bench :: pub fn black_box[T](value: T) -> T",
        "cell :: pub fn Cell.new[T](value: T) -> Cell[T]",
        "cell :: pub fn Cell.get[T](self) -> T",
        "cell :: pub fn Cell.set[T](&mut self, value: T)",
        "cell :: pub fn Cell.replace[T](&mut self, value: T) -> T",
        "cell :: pub fn Cell.swap[T](&mut self, other: &mut Cell[T])",
        "cell :: pub fn RefCell.new[T](value: T) -> RefCell[T]",
        "cell :: pub fn RefCell.borrow[T](&mut self) -> Ref[T]",
        "cell :: pub fn RefCell.borrow_mut[T](&mut self) -> RefMut[T]",
        "cell :: pub fn RefCell.try_borrow[T](&mut self) -> Option[Ref[T]]",
        "cell :: pub fn RefCell.try_borrow_mut[T](&mut self) -> Option[RefMut[T]]",
        "cell :: pub fn RefCell.replace[T](&mut self, value: T) -> T",
        "cell :: pub fn Ref.release[T](self)",
        "cell :: pub fn Ref.get[T](self) -> T",
        "cell :: pub fn RefMut.release[T](self)",
        "cell :: pub fn RefMut.get[T](self) -> T",
        "cell :: pub fn RefMut.set[T](self, value: T)",
        "char :: pub fn is_alphabetic(c: Char) -> Bool",
        "char :: pub fn is_alphanumeric(c: Char) -> Bool",
        "char :: pub fn is_ascii(c: Char) -> Bool",
        "char :: pub fn is_control(c: Char) -> Bool",
        "char :: pub fn is_digit(c: Char) -> Bool",
        "char :: pub fn is_lowercase(c: Char) -> Bool",
        "char :: pub fn is_uppercase(c: Char) -> Bool",
        "char :: pub fn is_numeric(c: Char) -> Bool",
        "char :: pub fn is_punctuation(c: Char) -> Bool",
        "char :: pub fn is_whitespace(c: Char) -> Bool",
        "char :: pub fn to_lowercase(c: Char) -> Char",
        "char :: pub fn to_uppercase(c: Char) -> Char",
        "char :: pub fn to_digit(c: Char, radix: Int) -> Option[Int]",
        "char :: pub fn from_digit(n: Int, radix: Int) -> Option[Char]",
        "char :: pub fn len_utf8(c: Char) -> Int",
        "char :: pub fn encode_utf8(c: Char, buf: &mut Vec[UInt8])",
        "cmp :: pub fn Ordering.reverse(self) -> Ordering",
        "cmp :: pub fn Ordering.then(self, other: Ordering) -> Ordering",
        "cmp :: pub fn Ordering.then_with(self, f: fn() -> Ordering) -> Ordering",
        "cmp :: pub fn min[T: Ord](a: T, b: T) -> T",
        "cmp :: pub fn max[T: Ord](a: T, b: T) -> T",
        "cmp :: pub fn clamp[T: Ord](value: T, min_val: T, max_val: T) -> T",
        "cmp :: pub fn min_by[T](a: T, b: T, compare: fn(&T, &T) -> Ordering) -> T",
        "cmp :: pub fn max_by[T](a: T, b: T, compare: fn(&T, &T) -> Ordering) -> T",
        "cmp :: pub fn max_int(a: Int, b: Int) -> Int",
        "cmp :: pub fn min_int(a: Int, b: Int) -> Int",
        "cmp :: pub fn clamp_int(value: Int, min_val: Int, max_val: Int) -> Int",
        "cmp :: pub fn max_float(a: Float64, b: Float64) -> Float64",
        "cmp :: pub fn min_float(a: Float64, b: Float64) -> Float64",
        "cmp :: pub fn clamp_float(value: Float64, min_val: Float64, max_val: Float64) -> Float64",
        "cmp :: pub fn Reverse.new[T](value: T) -> Reverse[T]",
        "collections :: pub fn Vec[T].as_slice(self) -> Slice[T]",
        "collections :: pub fn Vec[T].as_mut_slice(self) -> Slice[T]",
        "compress :: pub fn GzipCompressor.new() -> GzipCompressor",
        "compress :: pub fn GzipCompressor.with_level(level: Int) -> GzipCompressor",
        "compress :: pub fn gzip_compress(data: &Vec[UInt8]) -> Result[Vec[UInt8], Str]",
        "compress :: pub fn gzip_compress_level(data: &Vec[UInt8], level: Int) -> Result[Vec[UInt8], Str]",
        "compress :: pub fn gzip_decompress(data: &Vec[UInt8]) -> Result[Vec[UInt8], Str]",
        "compress :: pub fn deflate_compress(data: &Vec[UInt8]) -> Result[Vec[UInt8], Str]",
        "compress :: pub fn deflate_decompress(data: &Vec[UInt8]) -> Result[Vec[UInt8], Str]",
        "compress :: pub fn deflate_compress_level(data: &Vec[UInt8], level: Int) -> Result[Vec[UInt8], Str]",
        "compress :: pub fn zlib_compress(data: &Vec[UInt8]) -> Result[Vec[UInt8], Str]",
        "compress :: pub fn zlib_compress_level(data: &Vec[UInt8], level: Int) -> Result[Vec[UInt8], Str]",
        "compress :: pub fn zlib_decompress(data: &Vec[UInt8]) -> Result[Vec[UInt8], Str]",
        "compress :: pub fn brotli_compress(data: &Vec[UInt8]) -> Result[Vec[UInt8], Str]",
        "compress :: pub fn brotli_compress_level(data: &Vec[UInt8], quality: Int) -> Result[Vec[UInt8], Str]",
        "compress :: pub fn brotli_decompress(data: &Vec[UInt8]) -> Result[Vec[UInt8], Str]",
        "compress :: pub fn lz4_compress(data: &Vec[UInt8]) -> Result[Vec[UInt8], Str]",
        "compress :: pub fn lz4_decompress(data: &Vec[UInt8]) -> Result[Vec[UInt8], Str]",
        "compress :: pub fn snappy_compress(data: &Vec[UInt8]) -> Result[Vec[UInt8], Str]",
        "compress :: pub fn snappy_decompress(data: &Vec[UInt8]) -> Result[Vec[UInt8], Str]",
        "compress :: pub fn compression_ratio(original: Int, compressed: Int) -> Float64",
        "compress :: pub fn is_compressed(data: &Vec[UInt8]) -> Bool",
        "compress :: pub fn detect_format(data: &Vec[UInt8]) -> Str",
        "contracts :: pub fn verify_invariants[T](value: &T) -> Vec[ContractCheckResult]",
        "contracts :: pub fn verify_function_contracts(func: Str, args: Map[Str, Str]) -> Vec[ContractCheckResult]",
        "contracts :: pub fn check_invariant[T](value: &T, invariant: Str) -> ContractCheckResult",
        "contracts :: pub fn build_contract_index() -> ContractIndex",
        "contracts :: pub fn get_function_contracts(name: Str) -> Option<Vec[FunctionContracts>>",
        "contracts :: pub fn get_type_contracts(name: Str) -> Option<Vec<TypeContracts>>",
        "contracts :: pub fn find_functions_using_type(type_name: Str) -> Vec[Str]",
        "contracts :: pub fn find_invariants_using_field(type_name: Str, field_name: Str) -> Vec[ContractClause]",
        "contracts :: pub fn export_contracts_json() -> Str",
        "contracts :: pub fn export_contracts_markdown() -> Str",
        "contracts :: pub fn export_contracts_openapi() -> Str",
        "contracts :: pub fn reset_contract_coverage()",
        "contracts :: pub fn record_contract_hit(clause: ContractClause, input_values: Map[Str, Str])",
        "contracts :: pub fn get_contract_coverage() -> Map[Str, Bool]",
        "contracts :: pub fn get_uncovered_contracts() -> Vec[ContractClause]",
        "contracts :: pub fn coverage_percentage() -> Float64",
        "contracts :: pub fn can_compose(f_requires: Vec[ContractClause], g_ensures: Vec[ContractClause]) -> Str",
        "contracts :: pub fn verify_chain(fns: Vec<Str>) -> Result[Unit, Vec[ContractCheckResult]]",
        "contracts :: pub fn total_contracts() -> Int",
        "contracts :: pub fn total_requires() -> Int",
        "contracts :: pub fn total_ensures() -> Int",
        "contracts :: pub fn total_invariants() -> Int",
        "contracts :: pub fn functions_with_contracts() -> Int",
        "contracts :: pub fn types_with_invariants() -> Int",
        "contracts :: pub fn contract_density() -> Float64",
        "contracts :: pub fn ContractIndex.none(self) -> Bool",
        "contracts :: pub fn ContractIndex.is_sorted(self) -> Bool",
        "contracts :: pub fn ContractIndex.contains_fn(self, name: Str) -> Bool",
        "contracts :: pub fn ContractIndex.contains_type(self, name: Str) -> Bool",
        "contracts :: pub fn ContractIndex.all_clauses(self) -> Vec[ContractClause]",
        "contracts :: pub fn ContractIndex.filter_nonempty(self) -> ContractIndex",
        "contracts :: pub fn any_contracts() -> Bool",
        "contracts :: pub fn fn_has_contracts(name: Str) -> Bool",
        "contracts :: pub fn type_has_invariants(name: Str) -> Bool",
        "convert :: pub fn identity[T](x: T) -> T",
        "convert :: pub fn int_to_float(n: Int) -> Float64",
        "convert :: pub fn float_to_int(f: Float64) -> Int",
        "convert :: pub fn int_to_string(n: Int) -> Str",
        "convert :: pub fn float_to_string(f: Float64) -> Str",
        "convert :: pub fn bool_to_string(b: Bool) -> Str",
        "convert :: pub fn char_to_int(c: Char) -> Int",
        "convert :: pub fn int_to_char(n: Int) -> Option[Char]",
        "core :: pub fn to_float_from_str(s: Str) -> Result[Float64, Str]",
        "core :: pub fn is_sorted[T: Ord](items: &Slice[T]) -> Bool",
        "core :: pub fn contains[T: Eq](items: &Slice[T], value: T) -> Bool",
        "core :: pub fn Box[T].deref(self) -> &T",
        "core :: pub fn Box[T].deref_mut(self) -> &mut T",
        "core :: pub fn Box[T].as_ref(self) -> &T",
        "core :: pub fn Box[T].as_mut(self) -> &mut T",
        "core :: pub fn Str.as_ref(self) -> &Str",
        "core :: pub fn Str.as_bytes(self) -> &Slice[UInt8]",
        "core :: pub fn Cow[T: Clone].is_borrowed(self) -> Bool",
        "core :: pub fn Cow[T: Clone].is_owned(self) -> Bool",
        "core :: pub fn Cow[T: Clone].to_mut(self) -> &mut T",
        "core :: pub fn Cow[T: Clone].into_owned(self) -> T",
        "core :: pub fn Cow[T: Clone].borrow(self) -> &T",
        "core :: pub fn Int.from(value: Float64) -> Int",
        "core :: pub fn Float64.into(self) -> Int",
        "core :: pub fn Float64.from(value: Int) -> Float64",
        "core :: pub fn Int.into(self) -> Float64",
        "core :: pub fn Str.from(value: Int) -> Str",
        "core :: pub fn Int.into(self) -> Str",
        "core :: pub fn Str.from(value: Float64) -> Str",
        "core :: pub fn Float64.into(self) -> Str",
        "core :: pub fn Str.from(value: Bool) -> Str",
        "core :: pub fn Bool.into(self) -> Str",
        "core :: pub fn Int.from(value: Bool) -> Int",
        "core :: pub fn Bool.into(self) -> Int",
        "core :: pub fn Int.from(value: Char) -> Int",
        "core :: pub fn Char.into(self) -> Int",
        "core :: pub fn Char.from(value: Int) -> Char",
        "core :: pub fn Int.into(self) -> Char",
        "core :: pub fn MaybeUninit[T].uninit() -> MaybeUninit[T]",
        "core :: pub fn MaybeUninit[T].new(value: T) -> MaybeUninit[T]",
        "core :: pub fn MaybeUninit[T].assume_init(self) -> T",
        "core :: pub fn MaybeUninit[T].write(self, value: T)",
        "crypto :: pub fn sha256(data: &Vec[UInt8]) -> Vec[UInt8]",
        "crypto :: pub fn sha256_accelerated(data: &Vec[UInt8]) -> Vec[UInt8]",
        "crypto :: pub fn sha256_hex(data: &Vec[UInt8]) -> Str",
        "crypto :: pub fn sha512(data: &Vec[UInt8]) -> Vec[UInt8]",
        "crypto :: pub fn md5(data: &Vec[UInt8]) -> Vec[UInt8]",
        "crypto :: pub fn blake3(data: &Vec[UInt8]) -> Vec[UInt8]",
        "crypto :: pub fn hmac_sha256(key: &Vec[UInt8], data: &Vec[UInt8]) -> Vec[UInt8]",
        "crypto :: pub fn aes_encrypt(key: &Vec[UInt8], plaintext: &Vec[UInt8]) -> Result[Vec[UInt8], Str]",
        "crypto :: pub fn aes_decrypt(key: &Vec[UInt8], ciphertext: &Vec[UInt8]) -> Result<Vec[UInt8], Str>",
        "crypto :: pub fn aes_encrypt_gcm(key: &Vec[UInt8], nonce: &Vec[UInt8], plaintext: &Vec[UInt8], aad: &Vec[UInt8]) -> Result<(Vec[UInt8], Vec[UInt8]), Str>",
        "crypto :: pub fn aes_decrypt_gcm(key: &Vec[UInt8], nonce: &Vec[UInt8], ciphertext: &Vec[UInt8], tag: &Vec[UInt8], aad: &Vec[UInt8]) -> Result<Vec[UInt8], Str>",
        "crypto :: pub fn generate_rsa_keypair(bits: Int) -> Result<KeyPair, Str>",
        "crypto :: pub fn rsa_encrypt(public_key: &Vec[UInt8], data: &Vec[UInt8]) -> Result<Vec[UInt8], Str>",
        "crypto :: pub fn rsa_decrypt(private_key: &Vec[UInt8], data: &Vec[UInt8]) -> Result<Vec[UInt8], Str>",
        "crypto :: pub fn rsa_sign(private_key: &Vec[UInt8], data: &Vec[UInt8]) -> Result<Vec[UInt8], Str>",
        "crypto :: pub fn rsa_verify(public_key: &Vec[UInt8], data: &Vec[UInt8], signature: &Vec[UInt8]) -> Result<Bool, Str>",
        "crypto :: pub fn pbkdf2(password: &Str, salt: &Vec[UInt8], iterations: Int, key_len: Int) -> Vec[UInt8]",
        "crypto :: pub fn argon2(password: &Str, salt: &Vec[UInt8], memory: Int, iterations: Int, parallelism: Int) -> Vec[UInt8]",
        "crypto :: pub fn secure_random_bytes(count: Int) -> Vec[UInt8]",
        "crypto :: pub fn constant_time_compare(a: &Vec[UInt8], b: &Vec[UInt8]) -> Bool",
        "encoding :: pub fn base64_encode(data: &Vec[UInt8]) -> Str",
        "encoding :: pub fn base64_decode(encoded: Str) -> Result[Vec[UInt8], Str]",
        "encoding :: pub fn base64url_encode(data: &Vec[UInt8]) -> Str",
        "encoding :: pub fn base64url_decode(encoded: Str) -> Result[Vec[UInt8], Str]",
        "encoding :: pub fn hex_encode(data: &Vec[UInt8]) -> Str",
        "encoding :: pub fn hex_decode(encoded: Str) -> Result[Vec[UInt8], Str]",
        "encoding :: pub fn hex_encode_upper(data: &Vec[UInt8]) -> Str",
        "encoding :: pub fn url_encode(data: Str) -> Str",
        "encoding :: pub fn url_decode(encoded: Str) -> Result[Str, Str]",
        "encoding :: pub fn percent_encode(data: Str) -> Str",
        "encoding :: pub fn percent_decode(encoded: Str) -> Result[Str, Str]",
        "encoding :: pub fn utf8_encode(s: Str) -> Vec[UInt8]",
        "encoding :: pub fn utf8_decode(data: &Vec[UInt8]) -> Result[Str, Str]",
        "encoding :: pub fn utf8_valid(data: &Vec[UInt8]) -> Bool",
        "encoding :: pub fn utf8_char_len(first_byte: UInt8) -> Int",
        "encoding :: pub fn binary_to_text(data: &Vec[UInt8], format: Int) -> Str",
        "encoding :: pub fn text_to_binary(text: Str, format: Int) -> Result[Vec[UInt8], Str]",
        "env :: pub fn get_var(name: Str) -> Result<Str, Str>",
        "env :: pub fn var_opt(name: Str) -> Option<Str>",
        "env :: pub fn set_var(name: Str, value: Str)",
        "env :: pub fn remove_var(name: Str)",
        "env :: pub fn vars() -> Vec<(Str, Str)>",
        "env :: pub fn args() -> Vec<Str>",
        "env :: pub fn args_os() -> Vec<Str>",
        "env :: pub fn current_exe() -> Result<Str, Str>",
        "env :: pub fn current_dir() -> Result<Str, Str>",
        "env :: pub fn set_current_dir(path: Str) -> Result<Unit, Str>",
        "env :: pub fn temp_dir() -> Str",
        "env :: pub fn home_dir() -> Option<Str>",
        "env :: pub fn data_dir() -> Option<Str>",
        "env :: pub fn cache_dir() -> Option<Str>",
        "env :: pub fn config_dir() -> Option<Str>",
        "env :: pub fn executable_dir() -> Option<Str>",
        "env :: pub fn join_paths(a: Str, b: Str) -> Str",
        "env :: pub fn path_separator() -> Str",
        "error :: pub fn Error.chain(self) -> ErrorChain",
        "error :: pub fn ErrorChain.display(self) -> Str",
        "error :: pub fn wrap_error[T, E](result: Result[T, E], context: Str) -> Result[T, Str]",
        "error :: pub fn context[T, E](result: Result[T, E], msg: Str) -> Result[T, Str]",
        "error :: pub fn capture_backtrace() -> Backtrace",
        "error :: pub fn Backtrace.display(self) -> Str",
        "ffi :: pub fn alloc(size: Int) -> *UInt8",
        "ffi :: pub fn free(ptr: *UInt8)",
        "ffi :: pub fn memcpy(dest: *UInt8, src: *UInt8, size: Int)",
        "ffi :: pub fn safe_ptr_alloc(size: Int) -> Result[SafePtr, Str]",
        "ffi :: pub fn safe_ptr_from_raw(ptr: *UInt8, size: Int) -> Result[SafePtr, Str]",
        "ffi :: pub fn safe_ptr_free(ptr: SafePtr)",
        "ffi :: pub fn safe_ptr_read_byte(ptr: &SafePtr, offset: Int) -> Result[Int, Str]",
        "ffi :: pub fn safe_ptr_write_byte(ptr: &mut SafePtr, offset: Int, val: Int) -> Result[Unit, Str]",
        "ffi :: pub fn safe_ptr_read_i32(ptr: &SafePtr, offset: Int) -> Result[Int, Str]",
        "ffi :: pub fn safe_ptr_write_i32(ptr: &mut SafePtr, offset: Int, val: Int) -> Result[Unit, Str]",
        "ffi :: pub fn safe_ptr_read_f32(ptr: &SafePtr, offset: Int) -> Result[Float32, Str]",
        "ffi :: pub fn safe_ptr_write_f32(ptr: &mut SafePtr, offset: Int, val: Float32) -> Result[Unit, Str]",
        "ffi :: pub fn buffer_new(capacity: Int) -> Result[FFIBuffer, Str]",
        "ffi :: pub fn buffer_write(buf: &mut FFIBuffer, data: &Vec[Int]) -> Result[Int, Str]",
        "ffi :: pub fn buffer_read(buf: &FFIBuffer, offset: Int, len: Int) -> Result[Vec[Int], Str]",
        "ffi :: pub fn buffer_clear(buf: &mut FFIBuffer)",
        "ffi :: pub fn buffer_len(buf: &FFIBuffer) -> Int",
        "ffi :: pub fn buffer_is_empty(buf: &FFIBuffer) -> Bool",
        "ffi :: pub fn ffi_check(code: Int, msg: Str) -> Result[Int, FFIError]",
        "ffi :: pub fn ffi_check_ptr(ptr: *UInt8, msg: Str) -> Result[*UInt8, FFIError]",
        "ffi :: pub fn ffi_check_nonzero(code: Int, msg: Str) -> Result[Int, FFIError]",
        "ffi :: pub fn ffi_ok() -> Int",
        "ffi :: pub fn ffi_error(code: Int, msg: Str) -> FFIError",
        "ffi :: pub fn write_u32_at(dest: Int, offset: Int, value: Int)",
        "ffi :: pub fn write_u64_at(dest: Int, offset: Int, value: Int)",
        "ffi :: pub fn write_f32_at(dest: Int, offset: Int, value: Float32)",
        "ffi :: pub fn write_str_at(dest: Int, offset: Int, s: Str)",
        "ffi :: pub fn size_of[T]() -> Int",
        "ffi :: pub fn align_of[T]() -> Int",
        "ffi :: pub fn extern_c(name: Str) -> Int",
        "fmt :: pub fn Formatter.new() -> Formatter",
        "fmt :: pub fn Formatter.write_str(self, s: Str) -> Result[Unit, FmtError]",
        "fmt :: pub fn Formatter.write_int(self, n: Int) -> Result[Unit, FmtError]",
        "fmt :: pub fn Formatter.write_float(self, f: Float64) -> Result[Unit, FmtError]",
        "fmt :: pub fn Formatter.write_bool(self, b: Bool) -> Result[Unit, FmtError]",
        "fmt :: pub fn Formatter.finish(self) -> Str",
        "fmt :: pub fn Int.to_str() -> Str",
        "fmt :: pub fn Float64.to_str() -> Str",
        "fmt :: pub fn Bool.to_str() -> Str",
        "fmt :: pub fn Str.to_str() -> Str",
        "fmt :: pub fn format1[T](fmt: Str, arg: T) -> Str",
        "fmt :: pub fn format2[T, U](fmt: Str, arg1: T, arg2: U) -> Str",
        "fmt :: pub fn format3[T, U, V](fmt: Str, arg1: T, arg2: U, arg3: V) -> Str",
        "fmt :: pub fn print(s: Str)",
        "fmt :: pub fn println(s: Str)",
        "hash :: pub fn DefaultHasher.new() -> DefaultHasher",
        "hash :: pub fn DefaultHasher.write(self, bytes: &Vec[UInt8])",
        "hash :: pub fn DefaultHasher.write_int(self, n: Int)",
        "hash :: pub fn DefaultHasher.write_str(self, s: Str)",
        "hash :: pub fn DefaultHasher.finish(self) -> Int",
        "hash :: pub fn Int.hash(self) -> UInt64",
        "hash :: pub fn Bool.hash(self) -> UInt64",
        "hash :: pub fn hash_value[T: Hash](value: T) -> Int",
        "hash :: pub fn hash_combine(seed: Int, hash: Int) -> Int",
        "hash :: pub fn hash[T: Hash](value: T) -> UInt64",
        "hash :: pub fn sip_hash(data: &Vec[UInt8]) -> UInt64",
        "io :: pub fn print(msg: Str)",
        "io :: pub fn println(msg: Str)",
        "io :: pub fn read_line() -> Str",
        "io :: pub fn read_int() -> Result[Int, Str]",
        "io :: pub fn read_float() -> Result[Float64, Str]",
        "io :: pub fn read_file(path: Str) -> Result[Str, IOError]",
        "io :: pub fn write_file(path: Str, content: Str) -> Result[Unit, IOError]",
        "io :: pub fn append_file(path: Str, content: Str) -> Result[Unit, IOError]",
        "io :: pub fn file_exists(path: Str) -> Bool",
        "io :: pub fn is_dir(path: Str) -> Bool",
        "io :: pub fn create_dir(path: Str) -> Result[Unit, IOError]",
        "io :: pub fn list_dir(path: Str) -> Result[Vec[Str], IOError]",
        "io :: pub fn remove_file(path: Str) -> Result[Unit, IOError]",
        "io :: pub fn copy_file(src: Str, dst: Str) -> Result[Unit, IOError]",
        "io :: pub fn rename(src: Str, dst: Str) -> Result[Unit, IOError]",
        "io :: pub fn exit(code: Int)",
        "io :: pub fn args() -> Vec[Str]",
        "io :: pub fn env_var(name: Str) -> Option[Str]",
        "io :: pub fn time_now() -> Int",
        "io :: pub fn sleep(ms: Int)",
        "io :: pub fn BufReader.new(reader: Int) -> BufReader",
        "io :: pub fn BufReader.read_line(self, buf: &mut Str) -> Result[Int, IOError]",
        "io :: pub fn BufReader.lines(self) -> Vec[Str]",
        "io :: pub fn BufWriter.new(writer: Int) -> BufWriter",
        "io :: pub fn metadata(path: Str) -> Result[Metadata, IOError]",
        "io :: pub fn set_permissions(path: Str, perm: Int) -> Result[Unit, IOError]",
        "io :: pub fn stdin() -> Int",
        "io :: pub fn stdout() -> Int",
        "io :: pub fn stderr() -> Int",
        "io :: pub fn Cursor.new(data: Vec[UInt8]) -> Cursor",
        "io :: pub fn Cursor.into_inner(self) -> Vec[UInt8]",
        "io :: pub fn join_paths(base: Str, child: Str) -> Str",
        "io :: pub fn parent_path(path: Str) -> Option[Str]",
        "io :: pub fn file_name(path: Str) -> Option[Str]",
        "io :: pub fn extension(path: Str) -> Option[Str]",
        "io :: pub fn is_absolute(path: Str) -> Bool",
        "iter :: pub fn range(start: Int, end: Int) -> Range",
        "iter :: pub fn range_inclusive(start: Int, end: Int) -> RangeInclusive",
        "iter :: pub fn Range.next(self) -> Option[Int]",
        "iter :: pub fn Range.len(self) -> Int",
        "iter :: pub fn Range.contains(self, x: Int) -> Bool",
        "iter :: pub fn Range.sum(self) -> Int",
        "iter :: pub fn Range.product(self) -> Int",
        "iter :: pub fn RangeInclusive.next(self) -> Option[Int]",
        "iter :: pub fn Iterator[T].step_by(self, step: Int) -> StepByIter[T]",
        "iter :: pub fn Iterator[T].step_by(self, step: Int) -> StepByIter[T]",
        "iter :: pub fn Iterator[T].step_by(self, step: Int) -> StepByIter[T]",
        "iter :: pub fn Iterator[T].step_by(self, step: Int) -> StepByIter[T]",
        "iter :: pub fn Iterator[T].step_by(self, step: Int) -> StepByIter[T]",
        "iter :: pub fn Iterator[T].step_by(self, step: Int) -> StepByIter[T]",
        "iter :: pub fn Iterator[T].step_by(self, step: Int) -> StepByIter[T]",
        "iter :: pub fn Iterator[T].step_by(self, step: Int) -> StepByIter[T]",
        "iter :: pub fn Iterator[T].step_by(self, step: Int) -> StepByIter[T]",
        "iter :: pub fn Iterator[T].step_by(self, step: Int) -> StepByIter[T]",
        "iter :: pub fn Iterator[T].step_by(self, step: Int) -> StepByIter[T]",
        "iter :: pub fn Iterator[T].step_by(self, step: Int) -> StepByIter[T]",
        "iter :: pub fn Iterator[T].step_by(self, step: Int) -> StepByIter[T]",
        "iter :: pub fn Iterator[T].step_by(self, step: Int) -> StepByIter[T]",
        "iter :: pub fn Iterator[T].step_by(self, step: Int) -> StepByIter[T]",
        "iter :: pub fn Iterator[T].step_by(self, step: Int) -> StepByIter[T]",
        "iter :: pub fn Iterator[T].step_by(self, step: Int) -> StepByIter[T]",
        "iter :: pub fn Iterator[T].step_by(self, step: Int) -> StepByIter[T]",
        "iter :: pub fn Iterator[T].step_by(self, step: Int) -> StepByIter[T]",
        "iter :: pub fn MapIter[T, U].next(self) -> Option[U]",
        "iter :: pub fn FilterIter[T].next(self) -> Option[T]",
        "iter :: pub fn EnumerateIter[T].next(self) -> Option[(Int, T)]",
        "iter :: pub fn TakeIter[T].next(self) -> Option[T]",
        "iter :: pub fn SkipIter[T].next(self) -> Option[T]",
        "iter :: pub fn ChainIter[T, U].next(self) -> Option[T]",
        "iter :: pub fn ZipIter[T, U].next(self) -> Option[(T, U)]",
        "iter :: pub fn Iterator[T].step_by(self, step: Int) -> StepByIter[T]",
        "iter :: pub fn StepByIter[T].next(self) -> Option[T]",
        "iter :: pub fn Iterator[T].take_while(self, predicate: fn(&T) -> Bool) -> TakeWhileIter[T]",
        "iter :: pub fn TakeWhileIter[T].next(self) -> Option[T]",
        "iter :: pub fn Iterator[T].skip_while(self, predicate: fn(&T) -> Bool) -> SkipWhileIter[T]",
        "iter :: pub fn SkipWhileIter[T].next(self) -> Option[T]",
        "iter :: pub fn Iterator[T].inspect(self, f: fn(&T)) -> InspectIter[T]",
        "iter :: pub fn InspectIter[T].next(self) -> Option[T]",
        "log :: pub fn trace(msg: Str)",
        "log :: pub fn debug(msg: Str)",
        "log :: pub fn info(msg: Str)",
        "log :: pub fn warn(msg: Str)",
        "log :: pub fn error(msg: Str)",
        "log :: pub fn fatal(msg: Str)",
        "log :: pub fn trace_with(msg: Str, data: Map[Str, Str])",
        "log :: pub fn debug_with(msg: Str, data: Map[Str, Str])",
        "log :: pub fn info_with(msg: Str, data: Map[Str, Str])",
        "log :: pub fn warn_with(msg: Str, data: Map[Str, Str])",
        "log :: pub fn error_with(msg: Str, data: Map[Str, Str])",
        "log :: pub fn set_level(level: LogLevel)",
        "log :: pub fn get_level() -> LogLevel",
        "log :: pub fn set_output(file: Str) -> Result[Unit, Str]",
        "log :: pub fn set_output_json(enabled: Bool)",
        "log :: pub fn set_output_color(enabled: Bool)",
        "log :: pub fn entries_since(instant: Instant) -> Vec[LogEntry]",
        "log :: pub fn clear_log()",
        "math :: pub fn sqrt(x: Float64) -> Float64",
        "math :: pub fn pow(base: Float64, exp: Float64) -> Float64",
        "math :: pub fn abs_int(x: Int) -> Int",
        "math :: pub fn abs_float(x: Float64) -> Float64",
        "math :: pub fn min_int(a: Int, b: Int) -> Int",
        "math :: pub fn max_int(a: Int, b: Int) -> Int",
        "math :: pub fn min_float(a: Float64, b: Float64) -> Float64",
        "math :: pub fn max_float(a: Float64, b: Float64) -> Float64",
        "math :: pub fn floor(x: Float64) -> Float64",
        "math :: pub fn ceil(x: Float64) -> Float64",
        "math :: pub fn round(x: Float64) -> Int",
        "math :: pub fn sin(x: Float64) -> Float64",
        "math :: pub fn cos(x: Float64) -> Float64",
        "math :: pub fn tan(x: Float64) -> Float64",
        "math :: pub fn asin(x: Float64) -> Float64",
        "math :: pub fn acos(x: Float64) -> Float64",
        "math :: pub fn atan(x: Float64) -> Float64",
        "math :: pub fn atan2(y: Float64, x: Float64) -> Float64",
        "math :: pub fn exp(x: Float64) -> Float64",
        "math :: pub fn ln(x: Float64) -> Float64",
        "math :: pub fn log10(x: Float64) -> Float64",
        "math :: pub fn log2(x: Float64) -> Float64",
        "math :: pub fn bit_and(a: Int, b: Int) -> Int",
        "math :: pub fn bit_or(a: Int, b: Int) -> Int",
        "math :: pub fn bit_xor(a: Int, b: Int) -> Int",
        "math :: pub fn bit_not(a: Int) -> Int",
        "math :: pub fn shl(a: Int, n: Int) -> Int",
        "math :: pub fn shr(a: Int, n: Int) -> Int",
        "math :: pub fn seed_rng(seed: Int)",
        "math :: pub fn random() -> Float64",
        "math :: pub fn random_range(min: Int, max: Int) -> Int",
        "math :: pub fn random_float() -> Float64",
        "math :: pub fn clamp(x: Float64, lo: Float64, hi: Float64) -> Float64",
        "math :: pub fn lerp(a: Float64, b: Float64, t: Float64) -> Float64",
        "math :: pub fn is_nan(x: Float64) -> Bool",
        "math :: pub fn is_inf(x: Float64) -> Bool",
        "math :: pub fn sqrt_pure(x: Float64) -> Float64",
        "math :: pub fn pow_pure(base: Float64, exp: Float64) -> Float64",
        "math :: pub fn abs_float_pure(x: Float64) -> Float64",
        "math :: pub fn floor_pure(x: Float64) -> Float64",
        "math :: pub fn ceil_pure(x: Float64) -> Float64",
        "math :: pub fn sin_pure(x: Float64) -> Float64",
        "math :: pub fn cos_pure(x: Float64) -> Float64",
        "math :: pub fn tan_pure(x: Float64) -> Float64",
        "math :: pub fn asin_pure(x: Float64) -> Float64",
        "math :: pub fn acos_pure(x: Float64) -> Float64",
        "math :: pub fn atan_pure(x: Float64) -> Float64",
        "math :: pub fn atan2_pure(y: Float64, x: Float64) -> Float64",
        "math :: pub fn exp_pure(x: Float64) -> Float64",
        "math :: pub fn ln_pure(x: Float64) -> Float64",
        "math :: pub fn log10_pure(x: Float64) -> Float64",
        "math :: pub fn log2_pure(x: Float64) -> Float64",
        "md5 :: pub fn md5(data: &Vec[Int]) -> Vec[Int]",
        "md5 :: pub fn md5_hex(data: &Vec[Int]) -> Str",
        "mem :: pub fn swap[T](a: &mut T, b: &mut T)",
        "mem :: pub fn replace[T](dest: &mut T, src: T) -> T",
        "mem :: pub fn take[T: Default](dest: &mut T) -> T",
        "mem :: pub fn drop[T](value: T)",
        "mem :: pub fn size_of[T]() -> Int",
        "mem :: pub fn align_of[T]() -> Int",
        "mem :: pub fn size_of_val[T](value: &T) -> Int",
        "mem :: pub fn min_align_of_val[T](value: &T) -> Int",
        "mem :: pub fn zeroed[T]() -> T",
        "mem :: pub fn uninitialized[T]() -> T",
        "mem :: pub fn ManuallyDrop.new[T](value: T) -> ManuallyDrop[T]",
        "mem :: pub fn ManuallyDrop.into_inner[T](self) -> T",
        "mem :: pub fn ManuallyDrop.take[T](self) -> T",
        "mem :: pub fn ManuallyDrop.drop[T](self)",
        "net :: pub fn tcp_connect(host: Str, port: Int) -> Result[TcpStream, NetError]",
        "net :: pub fn tcp_listen(host: Str, port: Int) -> Result[TcpListener, NetError]",
        "net :: pub fn TcpStream.read(self, buf: &mut Vec[UInt8]) -> Result[Int, NetError]",
        "net :: pub fn TcpStream.write(self, data: &Vec[UInt8]) -> Result[Int, NetError]",
        "net :: pub fn TcpStream.close(self) -> Result[Unit, NetError]",
        "net :: pub fn TcpListener.accept(self) -> Result[(TcpStream, Str), NetError]",
        "net :: pub fn http_get(url: Str) -> Result[HttpResponse, NetError]",
        "net :: pub fn http_post(url: Str, body: Str) -> Result[HttpResponse, NetError]",
        "net :: pub fn udp_bind(host: Str, port: Int) -> Result[UdpSocket, NetError]",
        "net :: pub fn UdpSocket.send_to(self, data: &Vec[UInt8], addr: Str, port: Int) -> Result[Int, NetError]",
        "net :: pub fn UdpSocket.recv_from(self, buf: &mut Vec[UInt8]) -> Result[(Int, Str, Int), NetError]",
        "net :: pub fn UdpSocket.close(self) -> Result[Unit, NetError]",
        "net :: pub fn resolve_host(hostname: Str) -> Result[Vec[Str], NetError]",
        "net :: pub fn local_addr(port: Int) -> Result[Str, NetError]",
        "net :: pub fn parse_url(url: Str) -> Result[UrlParts, NetError]",
        "num :: pub fn min_value[T: Bounded]() -> T",
        "num :: pub fn max_value[T: Bounded]() -> T",
        "num :: pub fn epsilon[T: Bounded]() -> T",
        "num :: pub fn gcd(a: Int, b: Int) -> Int",
        "num :: pub fn lcm(a: Int, b: Int) -> Int",
        "num :: pub fn is_power_of_two(n: Int) -> Bool",
        "num :: pub fn next_power_of_two(n: Int) -> Int",
        "num :: pub fn count_ones(n: Int) -> Int",
        "num :: pub fn count_zeros(n: Int) -> Int",
        "num :: pub fn leading_zeros(n: Int) -> Int",
        "num :: pub fn trailing_zeros(n: Int) -> Int",
        "num :: pub fn rotate_left(n: Int, k: Int) -> Int",
        "num :: pub fn rotate_right(n: Int, k: Int) -> Int",
        "num :: pub fn reverse_bits(n: Int) -> Int",
        "num :: pub fn to_be(n: Int) -> Int",
        "num :: pub fn to_le(n: Int) -> Int",
        "num :: pub fn from_be(n: Int) -> Int",
        "num :: pub fn from_le(n: Int) -> Int",
        "num :: pub fn is_finite(x: Float64) -> Bool",
        "num :: pub fn is_normal(x: Float64) -> Bool",
        "num :: pub fn classify(x: Float64) -> Int",
        "num :: pub fn floor(x: Float64) -> Int",
        "num :: pub fn ceil(x: Float64) -> Int",
        "num :: pub fn round(x: Float64) -> Int",
        "num :: pub fn trunc(x: Float64) -> Int",
        "num :: pub fn fract(x: Float64) -> Float64",
        "num :: pub fn recip(x: Float64) -> Float64",
        "num :: pub fn to_degrees(rad: Float64) -> Float64",
        "num :: pub fn to_radians(deg: Float64) -> Float64",
        "num :: pub fn hypot(x: Float64, y: Float64) -> Float64",
        "num :: pub fn saturating_add[T: Bounded + Ord + Add](a: T, b: T) -> T",
        "num :: pub fn saturating_sub[T: Bounded + Ord + Sub](a: T, b: T) -> T",
        "num :: pub fn saturating_mul[T: Bounded + Ord + Mul + Div](a: T, b: T) -> T",
        "num :: pub fn checked_add[T: Bounded + Ord + Add](a: T, b: T) -> Option[T]",
        "num :: pub fn checked_sub[T: Bounded + Ord + Sub](a: T, b: T) -> Option[T]",
        "num :: pub fn checked_mul[T: Bounded + Ord + Mul + Div](a: T, b: T) -> Option[T]",
        "num :: pub fn checked_div[T: Bounded + Eq + Div](a: T, b: T) -> Option[T]",
        "num :: pub fn wrapping_add[T: Bounded + Add](a: T, b: T) -> T",
        "num :: pub fn wrapping_sub[T: Bounded + Sub](a: T, b: T) -> T",
        "num :: pub fn wrapping_mul[T: Bounded + Mul](a: T, b: T) -> T",
        "num :: pub fn parse_int(s: Str) -> Result[Int, Str]",
        "num :: pub fn parse_float(s: Str) -> Result[Float64, Str]",
        "num :: pub fn parse_int_radix(s: Str, radix: Int) -> Result[Int, Str]",
        "os :: pub fn platform() -> Str",
        "os :: pub fn cpu_count() -> Int",
        "os :: pub fn total_memory() -> Int",
        "os :: pub fn free_memory() -> Int",
        "os :: pub fn env_set(name: Str, value: Str)",
        "os :: pub fn env_unset(name: Str)",
        "os :: pub fn current_dir() -> Str",
        "os :: pub fn set_current_dir(path: Str) -> Result[Unit, Str]",
        "os :: pub fn temp_dir() -> Str",
        "os :: pub fn home_dir() -> Option[Str]",
        "os :: pub fn ChildProcess.wait(self) -> Result[Int, Str]",
        "os :: pub fn ChildProcess.kill(self) -> Result[Unit, Str]",
        "os :: pub fn ChildProcess.id(self) -> Int",
        "os :: pub fn walk_dir(path: Str, callback: fn(Str, Metadata) -> Unit) -> Result[Unit, Str]",
        "os :: pub fn walk_dir_filtered(path: Str, pattern: Str, callback: fn(Str, Metadata) -> Unit) -> Result[Unit, Str]",
        "os :: pub fn watch_file(path: Str) -> Result[FileWatcher, Str]",
        "os :: pub fn watch_dir(path: Str, recursive: Bool) -> Result[FileWatcher, Str]",
        "os :: pub fn FileWatcher.poll(self) -> Result[Vec[FileEvent], Str]",
        "os :: pub fn FileWatcher.close(self)",
        "os :: pub fn on_signal(signal: Int, handler: fn(Int) -> Unit)",
        "os :: pub fn raise_signal(signal: Int)",
        "os :: pub fn create_pipe() -> Result[Pipe, Str]",
        "os :: pub fn Pipe.read(self, buf: &mut Vec[UInt8]) -> Result[Int, Str]",
        "os :: pub fn Pipe.write(self, data: &Vec[UInt8]) -> Result[Int, Str]",
        "os :: pub fn Pipe.close_read(self)",
        "os :: pub fn Pipe.close_write(self)",
        "os :: pub fn disk_free(path: Str) -> Result[Int, Str]",
        "os :: pub fn disk_total(path: Str) -> Result[Int, Str]",
        "os :: pub fn file_size_bytes(path: Str) -> Result[Int, Str]",
        "path :: pub fn Path.new(s: Str) -> Path",
        "path :: pub fn PathBuf.new() -> PathBuf",
        "path :: pub fn PathBuf.from(s: Str) -> PathBuf",
        "path :: pub fn Path.parent(self) -> Option<Path>",
        "path :: pub fn Path.file_name(self) -> Option<Str>",
        "path :: pub fn Path.extension(self) -> Option<Str>",
        "path :: pub fn Path.file_stem(self) -> Option<Str>",
        "path :: pub fn Path.is_absolute(self) -> Bool",
        "path :: pub fn Path.is_relative(self) -> Bool",
        "path :: pub fn Path.has_root(self) -> Bool",
        "path :: pub fn Path.components(self) -> Vec<Str>",
        "path :: pub fn Path.to_str(self) -> Str",
        "path :: pub fn Path.join(self, child: Str) -> PathBuf",
        "path :: pub fn Path.with_extension(self, ext: Str) -> PathBuf",
        "path :: pub fn Path.with_file_name(self, name: Str) -> PathBuf",
        "path :: pub fn Path.exists(self) -> Bool",
        "path :: pub fn Path.is_file(self) -> Bool",
        "path :: pub fn Path.is_dir(self) -> Bool",
        "path :: pub fn Path.metadata(self) -> Result<Metadata, Str>",
        "path :: pub fn Path.canonicalize(self) -> Result<PathBuf, Str>",
        "path :: pub fn Path.starts_with(self, base: Path) -> Bool",
        "path :: pub fn Path.ends_with(self, child: Path) -> Bool",
        "path :: pub fn PathBuf.push(&mut self, component: Str)",
        "path :: pub fn PathBuf.pop(&mut self) -> Bool",
        "path :: pub fn PathBuf.as_path(self) -> Path",
        "path :: pub fn PathBuf.clear(&mut self)",
        "path :: pub fn path_separator() -> Str",
        "ptr :: pub fn null[T]() -> *T",
        "ptr :: pub fn null_mut[T]() -> *mut T",
        "ptr :: pub fn dangling[T]() -> *T",
        "ptr :: pub fn is_null[T](ptr: *const T) -> Bool",
        "ptr :: pub fn read[T](ptr: *const T) -> T",
        "ptr :: pub fn write[T](ptr: *mut T, value: T)",
        "ptr :: pub fn read_volatile[T](ptr: *const T) -> T",
        "ptr :: pub fn write_volatile[T](ptr: *mut T, value: T)",
        "ptr :: pub fn swap[T](a: *mut T, b: *mut T)",
        "ptr :: pub fn replace[T](dest: *mut T, src: T) -> T",
        "ptr :: pub fn copy[T](src: *const T, dst: *mut T, count: Int)",
        "ptr :: pub fn copy_nonoverlapping[T](src: *const T, dst: *mut T, count: Int)",
        "ptr :: pub fn eq[T](a: *const T, b: *const T) -> Bool",
        "ptr :: pub fn offset[T](ptr: *const T, count: Int) -> *const T",
        "ptr :: pub fn wrapping_offset[T](ptr: *const T, count: Int) -> *const T",
        "ptr :: pub fn add[T](ptr: *const T, count: Int) -> *const T",
        "ptr :: pub fn sub[T](ptr: *const T, count: Int) -> *const T",
        "ptr :: pub fn from_ref[T](r: &T) -> *const T",
        "ptr :: pub fn from_mut[T](r: &mut T) -> *mut T",
        "rand :: pub fn StdRng.new() -> StdRng",
        "rand :: pub fn StdRng.from_seed(seed: Int) -> StdRng",
        "rand :: pub fn random() -> Float64",
        "rand :: pub fn random_int(min: Int, max: Int) -> Int",
        "rand :: pub fn random_float(min: Float64, max: Float64) -> Float64",
        "rand :: pub fn random_bool() -> Bool",
        "rand :: pub fn random_bytes(count: Int) -> Vec[UInt8]",
        "rand :: pub fn sample_uniform(min: Float64, max: Float64) -> Float64",
        "rand :: pub fn sample_normal(mean: Float64, stddev: Float64) -> Float64",
        "rand :: pub fn sample_exponential(lambda: Float64) -> Float64",
        "rand :: pub fn sample_bernoulli(p: Float64) -> Bool",
        "rand :: pub fn sample_binomial(n: Int, p: Float64) -> Int",
        "rand :: pub fn sample_poisson(lambda: Float64) -> Int",
        "rand :: pub fn sample_gamma(shape: Float64, scale: Float64) -> Float64",
        "rand :: pub fn sample_beta(alpha: Float64, beta: Float64) -> Float64",
        "rand :: pub fn shuffle[T](items: &mut Vec[T])",
        "rand :: pub fn pick[T](items: &Vec[T]) -> Option<&T>",
        "rand :: pub fn pick_n[T](items: &Vec[T], n: Int) -> Vec<&T>",
        "rand :: pub fn weighted_pick[T](items: &Vec[T], weights: &Vec[Float64]) -> Option<&T>",
        "rand :: pub fn uuid_v4() -> Str",
        "rand :: pub fn uuid_v7() -> Str",
        "rand :: pub fn seed_from_entropy()",
        "rand :: pub fn seed_from_time()",
        "rand :: pub fn seed_from_value(seed: Int)",
        "rc :: pub fn Rc.new[T](value: T) -> Rc[T]",
        "rc :: pub fn Rc.clone[T](self) -> Rc[T]",
        "rc :: pub fn Rc.strong_count[T](self) -> Int",
        "rc :: pub fn Rc.weak_count[T](self) -> Int",
        "rc :: pub fn Rc.get[T](self) -> T",
        "rc :: pub fn Rc.ptr_eq[T, U](self, other: &Rc[U]) -> Bool",
        "rc :: pub fn Rc.downgrade[T](self) -> Weak[T]",
        "rc :: pub fn Rc.unwrap_or_clone[T: Clone](self) -> T",
        "rc :: pub fn Rc.drop[T](self)",
        "rc :: pub fn Rc[T].deref(self) -> &T",
        "rc :: pub fn Rc[T].as_ref(self) -> &T",
        "rc :: pub fn Weak.upgrade[T](self) -> Option[Rc[T]]",
        "rc :: pub fn Weak.strong_count[T](self) -> Int",
        "rc :: pub fn Weak.weak_count[T](self) -> Int",
        "rc :: pub fn Weak.drop[T](self)",
        "reflect :: pub fn type_count() -> Int",
        "reflect :: pub fn type_name_by_id(id: Int) -> Str",
        "reflect :: pub fn type_id_by_name(name: Str) -> Int",
        "reflect :: pub fn type_field_count(id: Int) -> Int",
        "reflect :: pub fn TypeId.of[T]() -> TypeId",
        "reflect :: pub fn type_name[T]() -> Str",
        "reflect :: pub fn type_size[T]() -> Int",
        "reflect :: pub fn type_align[T]() -> Int",
        "reflect :: pub fn downcast_ref[T: Any](value: &dyn Any) -> Option<&T>",
        "reflect :: pub fn downcast_mut[T: Any](value: &mut dyn Any) -> Option<&mut T>",
        "reflect :: pub fn reflect_type[T]() -> TypeInfo",
        "reflect :: pub fn type_info_by_name(name: Str) -> Option<TypeInfo>",
        "reflect :: pub fn all_types() -> Vec<TypeInfo>",
        "regex :: pub fn Regex.new(pattern: Str) -> Result<Regex, Str>",
        "regex :: pub fn Regex.is_match(self, text: Str) -> Bool",
        "regex :: pub fn Regex.find(self, text: Str) -> Option[Match]",
        "regex :: pub fn Regex.find_all(self, text: Str) -> Vec[Match]",
        "regex :: pub fn Regex.captures(self, text: Str) -> Option<Captures>",
        "regex :: pub fn Regex.replace(self, text: Str, replacement: Str) -> Str",
        "regex :: pub fn Regex.replace_all(self, text: Str, replacement: Str) -> Str",
        "regex :: pub fn Regex.split(self, text: Str) -> Vec[Str]",
        "regex :: pub fn Regex.match_count(self, text: Str) -> Int",
        "regex :: pub fn Captures.get(self, index: Int) -> Option[Match]",
        "regex :: pub fn Captures.get_named(self, name: Str) -> Option[Match]",
        "regex :: pub fn Captures.len(self) -> Int",
        "regex :: pub fn regex_escape(pattern: Str) -> Str",
        "regex :: pub fn is_valid_regex(pattern: Str) -> Bool",
        "serialize :: pub fn SerializeError.format_error() -> Str",
        "serialize :: pub fn detect_format(data: &Vec[UInt8]) -> Str",
        "serialize :: pub fn is_valid_json(data: Str) -> Bool",
        "serialize :: pub fn is_valid_bytes(data: &Vec[UInt8]) -> Bool",
        "serialize :: pub fn json_string(s: Str) -> Str",
        "serialize :: pub fn json_number(n: Float64) -> Str",
        "serialize :: pub fn json_bool(b: Bool) -> Str",
        "serialize :: pub fn json_null() -> Str",
        "serialize :: pub fn json_array(items: Vec[Str]) -> Str",
        "serialize :: pub fn json_object(pairs: Vec[(Str, Str)]) -> Str",
        "serialize :: pub fn to_json[T: Serialize](value: T) -> Result[Str, SerializeError]",
        "serialize :: pub fn from_json[T: Deserialize](s: Str) -> Result[T, SerializeError]",
        "serialize :: pub fn json_parse(data: Str) -> Result[JsonValue, SerializeError]",
        "serialize :: pub fn parse_json(s: Str) -> Result[JsonValue, SerializeError]",
        "serialize :: pub fn JsonValue.to_str(self) -> Str",
        "serialize :: pub fn JsonValue.get(self, key: Str) -> Option[JsonValue]",
        "serialize :: pub fn JsonValue.index(self, i: Int) -> Option[JsonValue]",
        "serialize :: pub fn little_endian() -> Bool",
        "serialize :: pub fn big_endian() -> Bool",
        "sha :: pub fn sha256_initial_h0() -> Int",
        "sha :: pub fn sha256_initial_h1() -> Int",
        "sha :: pub fn sha256_initial_h2() -> Int",
        "sha :: pub fn sha256_initial_h3() -> Int",
        "sha :: pub fn sha256_initial_h4() -> Int",
        "sha :: pub fn sha256_initial_h5() -> Int",
        "sha :: pub fn sha256_initial_h6() -> Int",
        "sha :: pub fn sha256_initial_h7() -> Int",
        "sha :: pub fn sha256_k(index: Int) -> Int",
        "sha :: pub fn sha256(data: &Vec[Int]) -> Vec[Int]",
        "sha :: pub fn sha256_hex(data: &Vec[Int]) -> Str",
        "sha :: pub fn sha256_hmac(data: &Vec[Int], key: &Vec[Int]) -> Vec[Int]",
        "sha :: pub fn sha512_initial_h0() -> Int",
        "sha :: pub fn sha512_initial_h1() -> Int",
        "sha :: pub fn sha512_initial_h2() -> Int",
        "sha :: pub fn sha512_initial_h3() -> Int",
        "sha :: pub fn sha512_initial_h4() -> Int",
        "sha :: pub fn sha512_initial_h5() -> Int",
        "sha :: pub fn sha512_initial_h6() -> Int",
        "sha :: pub fn sha512_initial_h7() -> Int",
        "sha :: pub fn sha512_k(index: Int) -> Int",
        "sha :: pub fn sha512(data: &Vec[Int]) -> Vec[Int]",
        "sha :: pub fn sha512_hex(data: &Vec[Int]) -> Str",
        "simd :: pub fn simd_supported() -> Bool",
        "simd :: pub fn has_sse() -> Bool",
        "simd :: pub fn has_avx() -> Bool",
        "simd :: pub fn has_avx2() -> Bool",
        "simd :: pub fn has_avx512() -> Bool",
        "simd :: pub fn has_neon() -> Bool",
        "simd :: pub fn Vec4f.new(x: Float32, y: Float32, z: Float32, w: Float32) -> Vec4f",
        "simd :: pub fn Vec4f.splat(value: Float32) -> Vec4f",
        "simd :: pub fn Vec4f.zero() -> Vec4f",
        "simd :: pub fn Vec4f.add(self, other: Vec4f) -> Vec4f",
        "simd :: pub fn Vec4f.sub(self, other: Vec4f) -> Vec4f",
        "simd :: pub fn Vec4f.mul(self, other: Vec4f) -> Vec4f",
        "simd :: pub fn Vec4f.div(self, other: Vec4f) -> Vec4f",
        "simd :: pub fn Vec4f.sqrt(self) -> Vec4f",
        "simd :: pub fn Vec4f.dot(self, other: Vec4f) -> Float32",
        "simd :: pub fn Vec4f.get(self, index: Int) -> Float32",
        "simd :: pub fn Vec4f.set(self, index: Int, value: Float32)",
        "simd :: pub fn Vec4f.len(self) -> Float32",
        "simd :: pub fn Vec4f.normalize(self) -> Vec4f",
        "simd :: pub fn Vec4f.cross3(self, other: Vec4f) -> Vec4f",
        "simd :: pub fn Vec4f.drop(self)",
        "simd :: pub fn Vec4f.add_scalar(self, other: Vec4f) -> Vec4f",
        "simd :: pub fn Vec4f.mul_scalar(self, other: Vec4f) -> Vec4f",
        "simd :: pub fn Vec8f.new(v0: Float32, v1: Float32, v2: Float32, v3: Float32, v4: Float32, v5: Float32, v6: Float32, v7: Float32) -> Vec8f",
        "simd :: pub fn Vec8f.add(self, other: Vec8f) -> Vec8f",
        "simd :: pub fn Vec8f.mul(self, other: Vec8f) -> Vec8f",
        "string :: pub fn str_len(s: Str) -> Int",
        "string :: pub fn str_concat(a: Str, b: Str) -> Str",
        "string :: pub fn str_slice(s: Str, start: Int, end: Int) -> Str",
        "string :: pub fn str_contains(s: Str, substr: Str) -> Bool",
        "string :: pub fn str_starts_with(s: Str, prefix: Str) -> Bool",
        "string :: pub fn str_ends_with(s: Str, suffix: Str) -> Bool",
        "string :: pub fn str_split(s: Str, delimiter: Str) -> Vec[Str]",
        "string :: pub fn str_trim(s: Str) -> Str",
        "string :: pub fn str_to_int(s: Str) -> Result[Int, Str]",
        "string :: pub fn str_to_float(s: Str) -> Result[Float64, Str]",
        "string :: pub fn str_upper(s: Str) -> Str",
        "string :: pub fn str_lower(s: Str) -> Str",
        "string :: pub fn format(fmt: Str) -> Str",
        "string :: pub fn format1(fmt: Str, arg: Str) -> Str",
        "string :: pub fn format2(fmt: Str, arg1: Str, arg2: Str) -> Str",
        "string :: pub fn byte_at(s: Str, pos: Int) -> UInt8",
        "string :: pub fn char_at(s: Str, pos: Int) -> Option[Char]",
        "string :: pub fn index_of(s: Str, substr: Str) -> Option[Int]",
        "string :: pub fn last_index_of(s: Str, substr: Str) -> Option[Int]",
        "string :: pub fn replace(s: Str, from: Str, to: Str) -> Str",
        "string :: pub fn lines(s: Str) -> Vec[Str]",
        "string :: pub fn words(s: Str) -> Vec[Str]",
        "string :: pub fn is_empty(s: Str) -> Bool",
        "string :: pub fn char_count(s: Str) -> Int",
        "string :: pub fn byte_count(s: Str) -> Int",
        "sync :: pub fn Mutex.new[T](value: T) -> Mutex[T]",
        "sync :: pub fn Mutex.lock[T](self) -> MutexGuard[T]",
        "sync :: pub fn Mutex.try_lock[T](self) -> Option[MutexGuard[T]]",
        "sync :: pub fn Mutex.into_inner[T](self) -> T",
        "sync :: pub fn MutexGuard.get[T](self) -> T",
        "sync :: pub fn MutexGuard.get_mut[T](self) -> T",
        "sync :: pub fn MutexGuard.drop[T](self)",
        "sync :: pub fn RwLock.new[T](data: T) -> RwLock[T]",
        "sync :: pub fn RwLock.read[T](self) -> ReadGuard[T]",
        "sync :: pub fn RwLock.write[T](self) -> WriteGuard[T]",
        "sync :: pub fn RwLock.try_read[T](self) -> Option[ReadGuard[T]]",
        "sync :: pub fn RwLock.try_write[T](self) -> Option[WriteGuard[T]]",
        "sync :: pub fn ReadGuard.get[T](self) -> T",
        "sync :: pub fn ReadGuard.drop[T](self)",
        "sync :: pub fn WriteGuard.get[T](self) -> T",
        "sync :: pub fn WriteGuard.get_mut[T](self) -> T",
        "sync :: pub fn WriteGuard.drop[T](self)",
        "sync :: pub fn Condvar.new() -> Condvar",
        "sync :: pub fn Condvar.wait[T](self, guard: MutexGuard[T]) -> MutexGuard[T]",
        "sync :: pub fn Condvar.notify_one(self)",
        "sync :: pub fn Condvar.notify_all(self)",
        "sync :: pub fn Once.new() -> Once",
        "sync :: pub fn Once.call_once(self, f: fn())",
        "sync :: pub fn Once.is_completed(self) -> Bool",
        "sync :: pub fn Barrier.new(n: Int) -> Barrier",
        "sync :: pub fn Barrier.wait(self)",
        "sync :: pub fn Arc.new[T](value: T) -> Arc[T]",
        "sync :: pub fn Arc.clone[T](self) -> Arc[T]",
        "sync :: pub fn Arc.get[T](self) -> T",
        "sync :: pub fn Arc.strong_count[T](self) -> Int",
        "sync :: pub fn Arc.ptr_eq[T, U](self, other: &Arc[U]) -> Bool",
        "sync :: pub fn Arc.drop[T](self)",
        "sync :: pub fn Arc[T].deref(self) -> &T",
        "sync :: pub fn Arc[T].as_ref(self) -> &T",
        "sync :: pub fn AtomicBool.new(val: Bool) -> AtomicBool",
        "sync :: pub fn AtomicBool.load(self) -> Bool",
        "sync :: pub fn AtomicBool.store(self, val: Bool)",
        "sync :: pub fn AtomicBool.swap(self, val: Bool) -> Bool",
        "sync :: pub fn AtomicBool.compare_exchange(self, current: Bool, new: Bool) -> Bool",
        "sync :: pub fn AtomicInt.new(val: Int) -> AtomicInt",
        "sync :: pub fn AtomicInt.load(self) -> Int",
        "sync :: pub fn AtomicInt.store(self, val: Int) -> AtomicInt",
        "sync :: pub fn AtomicInt.fetch_add(self, val: Int) -> Int",
        "sync :: pub fn AtomicInt.fetch_sub(self, val: Int) -> Int",
        "sync :: pub fn AtomicInt.swap(self, val: Int) -> Int",
        "sync :: pub fn AtomicInt.compare_exchange(self, current: Int, new: Int) -> Bool",
        "test :: pub fn assert(condition: Bool, name: Str) -> TestResult",
        "test :: pub fn assert_eq[T: Eq](expected: T, actual: T, name: Str) -> TestResult",
        "test :: pub fn assert_ne[T: Eq](expected: T, actual: T, name: Str) -> TestResult",
        "test :: pub fn assert_lt[T: Ord](left: T, right: T, name: Str) -> TestResult",
        "test :: pub fn assert_gt[T: Ord](left: T, right: T, name: Str) -> TestResult",
        "test :: pub fn assert_contains(haystack: Str, needle: Str, name: Str) -> TestResult",
        "test :: pub fn assert_ok[T, E](result: Result[T, E], name: Str) -> TestResult",
        "test :: pub fn assert_err[T, E](result: Result[T, E], name: Str) -> TestResult",
        "test :: pub fn assert_some[T](option: Option[T], name: Str) -> TestResult",
        "test :: pub fn assert_none[T](option: Option[T], name: Str) -> TestResult",
        "test :: pub fn assert_contract[T](value: T, predicate: fn(&T) -> Bool, name: Str) -> TestResult",
        "test :: pub fn run(test: fn() -> TestResult) -> Int",
        "test :: pub fn run_all(tests: Vec[fn() -> TestResult]) -> Int",
        "test :: pub fn run_filtered(tests: Vec[fn() -> TestResult], filter: Str) -> Int",
        "test :: pub fn format_results(results: Vec[TestResult]) -> Str",
        "test :: pub fn format_results_json(results: Vec[TestResult]) -> Str",
        "test :: pub fn bench(name: Str, f: fn()) -> TestResult",
        "thread :: pub fn spawn[T](f: fn() -> T) -> JoinHandle[T]",
        "thread :: pub fn spawn_with_name[T](name: Str, f: fn() -> T) -> JoinHandle[T]",
        "thread :: pub fn JoinHandle.join[T](self) -> Result[T, Str]",
        "thread :: pub fn JoinHandle.is_finished[T](self) -> Bool",
        "thread :: pub fn JoinHandle.thread[T](self) -> Thread",
        "thread :: pub fn JoinHandle.detach[T](self)",
        "thread :: pub fn Thread.current() -> Thread",
        "thread :: pub fn Thread.id(self) -> Int",
        "thread :: pub fn Thread.name(self) -> Option[Str]",
        "thread :: pub fn sleep_ms(ms: Int)",
        "thread :: pub fn sleep(ms: Int)",
        "thread :: pub fn yield_now()",
        "thread :: pub fn scope[T](f: fn(&Scope) -> T) -> T",
        "thread :: pub fn Scope.spawn[T](self, f: fn() -> T) -> JoinHandle[T]",
        "thread :: pub fn available_parallelism() -> Int",
        "thread :: pub fn hardware_threads() -> Int",
        "thread :: pub fn current_thread_id() -> Int",
        "time :: pub fn Duration.new(secs: Int, nanos: Int) -> Duration",
        "time :: pub fn Duration.from_secs(s: Int) -> Duration",
        "time :: pub fn Duration.from_secs_f64(secs: Float64) -> Duration",
        "time :: pub fn Duration.from_millis(ms: Int) -> Duration",
        "time :: pub fn Duration.from_micros(us: Int) -> Duration",
        "time :: pub fn Duration.from_nanos(ns: Int) -> Duration",
        "time :: pub fn Duration.as_secs(self) -> Int",
        "time :: pub fn Duration.as_millis(self) -> Int",
        "time :: pub fn Duration.as_micros(self) -> Int",
        "time :: pub fn Duration.as_nanos(self) -> Int",
        "time :: pub fn Duration.as_secs_f64(self) -> Float64",
        "time :: pub fn Duration.subsec_nanos(self) -> Int",
        "time :: pub fn Duration.add(self, other: Duration) -> Duration",
        "time :: pub fn Duration.sub(self, other: Duration) -> Duration",
        "time :: pub fn Duration.mul(self, factor: Int) -> Duration",
        "time :: pub fn Duration.div(self, divisor: Int) -> Duration",
        "time :: pub fn Duration.checked_add(self, other: Duration) -> Option[Duration]",
        "time :: pub fn Duration.checked_sub(self, other: Duration) -> Option[Duration]",
        "time :: pub fn Instant.now() -> Instant",
        "time :: pub fn Instant.elapsed(self) -> Duration",
        "time :: pub fn Instant.duration_since(self, earlier: Instant) -> Duration",
        "time :: pub fn Instant.add(self, d: Duration) -> Instant",
        "time :: pub fn Instant.sub(self, d: Duration) -> Instant",
        "time :: pub fn SystemTime.now() -> SystemTime",
        "time :: pub fn SystemTime.unix_epoch() -> SystemTime",
        "time :: pub fn SystemTime.duration_since(self, earlier: SystemTime) -> Result[Duration, Str]",
        "time :: pub fn SystemTime.secs_since_epoch(self) -> Int",
        "time :: pub fn DateTime.now() -> DateTime",
        "time :: pub fn DateTime.year(self) -> Int",
        "time :: pub fn DateTime.month(self) -> Int",
        "time :: pub fn DateTime.day(self) -> Int",
        "time :: pub fn DateTime.hour(self) -> Int",
        "time :: pub fn DateTime.minute(self) -> Int",
        "time :: pub fn DateTime.second(self) -> Int",
        "time :: pub fn DateTime.weekday(self) -> Int",
        "time :: pub fn utc_now() -> DateTime",
        "time :: pub fn local_now() -> DateTime",
        "time :: pub fn sleep(dur: Duration)",
        "time :: pub fn sleep_ms(ms: Int)",
        "time :: pub fn sleep_until(instant: Instant)",
];

/// Extract `pub fn` signatures from a stdlib module file using the same
/// normalization as the snapshot generator: strip contract clauses (keeping
/// a trailing brace), then capture each `pub fn` up to its opening brace.
/// Handles multi-line signatures and multiple fns on one physical line.
fn extract_signatures(path: &Path) -> Vec<String> {
    let content = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("read stdlib module {}: {e}", path.display()));
    let content = content.replace("\r\n", "\n");

    // Drop requires/ensures contract lines; preserve a trailing '{'.
    let mut kept = String::new();
    for line in content.lines() {
        let t = line.trim_start();
        if t.starts_with("requires:") || t.starts_with("ensures:") {
            if t.trim_end().ends_with('{') {
                kept.push_str("{\n");
            }
            continue;
        }
        kept.push_str(line);
        kept.push('\n');
    }

    // Scan for `pub fn` ... up to the next '{', the next 'pub fn', or a
    // comment line (compiler-intrinsic fns have no body brace).
    let mut out = Vec::new();
    let bytes: Vec<char> = kept.chars().collect();
    let mut i = 0;
    while i + 6 < bytes.len() {
        if bytes[i..i + 6].iter().collect::<String>() == "pub fn"
            && (i == 0 || bytes[i - 1] == '\n' || bytes[i - 1] == ' ' || bytes[i - 1] == '\t')
        {
            let mut j = i;
            while j < bytes.len() && bytes[j] != '{' {
                // Stop at a following 'pub fn' or a comment line.
                if j + 6 < bytes.len()
                    && bytes[j] == '\n'
                    && bytes[j + 1..j + 7].iter().collect::<String>() == "pub fn"
                {
                    break;
                }
                // Indentation-aware comment stop: the 2026-08-16 refactor
                // added `// note` lines BETWEEN the signature and the '{'
                // (e.g. crypto.xi aes_encrypt documents "no requires clauses"
                // under the sig). The old check required '/' immediately
                // after '\n' and missed two-space-indented comments, so the
                // comment text leaked into the extracted signature.
                if j + 2 < bytes.len() && bytes[j] == '\n' {
                    let mut k = j + 1;
                    while k < bytes.len() && (bytes[k] == ' ' || bytes[k] == '\t') {
                        k += 1;
                    }
                    if k + 1 < bytes.len() && bytes[k] == '/' && bytes[k + 1] == '/' {
                        break;
                    }
                }
                j += 1;
            }
            let sig: String = bytes[i..j].iter().collect();
            let sig = sig.split_whitespace().collect::<Vec<_>>().join(" ");
            if !sig.is_empty() {
                out.push(sig);
            }
            i = j;
        }
        i += 1;
    }
    out
}
/// Resolve a frozen SHORT module name ("aes", "alloc", ...) to its stdlib
/// file path. The 2026-08-16 layout refactor moved modules from flat files
/// (stdlib/xiom/aes.xi) into folders (stdlib/xiom/crypto/aes.xi), so the
/// flat join no longer works. Resolution order (Item B, 2026-08-17):
///   1. STDLIB_MANIFEST.md exact entry `xiom.<name>` (the frozen layout map)
///   2. STDLIB_MANIFEST.md unique last-segment match (`xiom.crypto.aes`)
///   3. Filesystem scan for `<name>.xi` under stdlib/xiom
fn resolve_module_path(root: &Path, module: &str) -> Option<std::path::PathBuf> {
    let manifest = root.join("docs").join("STDLIB_MANIFEST.md");
    let exact = format!("xiom.{module}");
    let mut suffix_match: Option<std::path::PathBuf> = None;
    if let Ok(text) = fs::read_to_string(&manifest) {
        for line in text.lines() {
            let parts: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
            if parts.len() >= 4
                && parts[1].starts_with("xiom.")
                && parts[2].starts_with("stdlib/xiom/")
                && parts[2].ends_with(".xi")
            {
                if parts[1] == exact {
                    // R31: the manifest table can lag module moves (e.g.
                    // xiom.rc -> memory/rc.xi -> rc/rc.xi). Only accept the
                    // manifest path when it actually exists; otherwise keep
                    // scanning and fall through to the filesystem layout.
                    let candidate = root.join(parts[2]);
                    if candidate.exists() {
                        return Some(candidate);
                    }
                    continue;
                }
                if suffix_match.is_none() && parts[1].rsplit('.').next() == Some(module) {
                    suffix_match = Some(root.join(parts[2]));
                }
            }
        }
    }
    if let Some(p) = suffix_match {
        if p.exists() {
            return Some(p);
        }
    }
    // R31: the module tree resolves through the shared stdlib root (checkout,
    // XIOM_STDLIB, installed lib); legacy <repo>/stdlib is the fallback.
    let base = xiom_graph::paths::stdlib_root()
        .map(|r| r.join("xiom"))
        .unwrap_or_else(|| root.join("stdlib").join("xiom"));
    let direct = base.join(module).join(format!("{module}.xi"));
    if direct.exists() {
        return Some(direct);
    }
    let flat = base.join(format!("{module}.xi"));
    if flat.exists() {
        return Some(flat);
    }
    // R49-1 (stdlib relay p_module_path_alias): moved modules live under a
    // path that differs from their DECLARED name (crypto/legacy/md5.xi
    // declares `xiom.crypto.md5`; the frozen short name is "md5"). The
    // manifest table lags those moves, so fall back to a one-time header
    // index of the whole stdlib tree: declared dotted name -> file.
    let index = stdlib_header_index(&base);
    if let Some(p) = index.get(&format!("xiom.{module}")) {
        return Some(p.clone());
    }
    if let Some(p) = index.get(module) {
        return Some(p.clone());
    }
    None
}

/// R49-1: one-time recursive `module <dotted>` header index of the stdlib
/// tree (declared full name and last segment both key the path).
fn stdlib_header_index(base: &Path) -> &'static std::collections::HashMap<String, std::path::PathBuf> {
    static IDX: std::sync::OnceLock<std::collections::HashMap<String, std::path::PathBuf>> =
        std::sync::OnceLock::new();
    IDX.get_or_init(|| {
        let mut map = std::collections::HashMap::new();
        fn walk(dir: &Path, map: &mut std::collections::HashMap<String, std::path::PathBuf>) {
            let Ok(entries) = fs::read_dir(dir) else { return };
            for e in entries.flatten() {
                let p = e.path();
                if p.is_dir() {
                    walk(&p, map);
                } else if p.extension().map_or(false, |x| x == "xi") {
                    let Ok(text) = fs::read_to_string(&p) else { continue };
                    for line in text.lines() {
                        let t = line.trim();
                        if t.is_empty() || t.starts_with("//") { continue; }
                        if let Some(rest) = t.strip_prefix("module ") {
                            let decl = rest.trim_end_matches(';').trim();
                            if !decl.is_empty() {
                                map.entry(decl.to_string()).or_insert_with(|| p.clone());
                                if let Some(last) = decl.rsplit('.').next() {
                                    map.entry(last.to_string()).or_insert_with(|| p.clone());
                                }
                            }
                        }
                        break;
                    }
                }
            }
        }
        walk(base, &mut map);
        map
    })
}

#[test]
fn stdlib_api_freeze_no_removals() {
    // R31: loud SKIP without a stdlib checkout; hard FAIL in CI.
    if xiom_graph::paths::stdlib_or_skip().is_none() {
        return;
    }
    let root = project_root();
    let mut missing: Vec<String> = Vec::new();

    for entry in FROZEN {
        let (module, sig) = entry.split_once(" :: ").unwrap();
        let Some(path) = resolve_module_path(root, module) else {
            missing.push(format!("{module} :: {sig}  [module file deleted]"));
            continue;
        };
        let signatures = extract_signatures(&path);
        if !signatures.iter().any(|s| s == sig) {
            missing.push(format!("{module} :: {sig}"));
        }
    }

    assert!(
        missing.is_empty(),
        "STDLIB API FREEZE VIOLATION - {} frozen signature(s) missing.\n\
         The stdlib is additive-only. To change an API, update the snapshot\n\
         intentionally and migrate all callers in the same commit.\n\n{}",
        missing.len(),
        missing.join("\n")
    );
}

#[test]
fn stdlib_api_freeze_all_modules_compile() {
    // R31: same skip/fail contract as the freeze scan itself.
    if xiom_graph::paths::stdlib_or_skip().is_none() {
        return;
    }
    // Belt-and-suspenders: the full stdlib must still compile together
    // (mirrors stdlib_tests, but keeps the freeze gate self-contained).
    let root = project_root();
    let modules: std::collections::HashSet<String> = FROZEN.iter()
        .filter_map(|e| e.split_once(" :: ").map(|(m, _)| m.to_string()))
        .collect();
    let mut program = String::new();
    for m in &modules { program.push_str(&format!("use xiom.{m};\n")); }
    program.push_str("fn main() -> Int { return 0; }\n");

    let tmp = std::env::temp_dir().join("xiom_api_freeze_full.xi");
    fs::write(&tmp, &program).expect("write temp program");

    let output = Command::new(xiom_path())
        .args(["--emit-ir", tmp.to_str().unwrap()])
        .current_dir(root)
        .output()
        .expect("failed to execute xiom");

    assert!(
        output.status.success(),
        "stdlib failed to compile together:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let _ = fs::remove_file(&tmp);
}
