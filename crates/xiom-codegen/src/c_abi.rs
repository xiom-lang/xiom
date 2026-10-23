// XIOM Codegen -- C ABI lowering plan for aggregate parameters/returns.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! m351 (REL-20261016-2031-bindings-3, B-33): target-dispatched C ABI
//! lowering for aggregates at C boundaries.
//!
//! XIOM's `%struct.X` values follow the INTERNAL convention everywhere
//! (by value, LLVM's naked aggregate lowering). A C boundary is different:
//! the MS x64 ABI passes aggregates of size 1/2/4/8 bytes as integers and
//! every other aggregate BY REFERENCE to a caller-owned copy; returns of
//! aggregates larger than 8 bytes use a hidden `sret` pointer. LLVM's naked
//! aggregate call lowering does NOT match that (a 16B record goes in
//! rcx:rdx and a 16B return in rax:rdx), so a C callee dereferences a scalar
//! as a pointer (0xC0000005).
//!
//! `CAbiPlan` is computed from the XIOM-value LLVM signature and applied at
//! the two C boundaries: extern `declare`s + their call operands, and
//! `Int as fn(..)` cast call sites (the dlsym shape). XIOM-internal fn
//! values keep the internal convention.
//!
//! Only x86_64 Windows is enabled. System V and AArch64 naked lowering is
//! left untouched until each is validated on the relevant target (the
//! report's explicit "do not regress SysV/AArch64" requirement).

/// How one C parameter carries an XIOM value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParamAbi {
    /// Pass the value directly in its LLVM type (scalars and pointers).
    Direct,
    /// Win64 aggregate of exactly 1/2/4/8 bytes: C passes the raw bytes as
    /// an integer of that width (`i8`/`i16`/`i32`/`i64`). LLVM's naked
    /// `%struct` lowering splits the fields into separate registers instead
    /// (e.g. `{float,float}` -> xmm0/xmm1, `{i32,i32}` -> ecx/edx), which
    /// does NOT match C's single-register coercion, so the call operand is
    /// loaded from the struct's bytes as the integer.
    CoerceInt(String),
    /// Win64 aggregate of any other size: store the value into a
    /// caller-owned stack slot and pass a pointer to it.
    IndirectCopy,
    /// System V MEMORY-class aggregate (>16 bytes): store the value into a
    /// caller-owned slot and pass the pointer with `byval` so LLVM copies it
    /// onto the outgoing stack in the C layout.
    IndirectCopyByVal,
}

impl Default for ParamAbi {
    fn default() -> Self {
        ParamAbi::Direct
    }
}

/// How a C return value carries the XIOM value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RetAbi {
    /// Return directly in the ABI return register(s).
    Direct,
    /// Win64 aggregate of exactly 1/2/4/8 bytes: C returns the raw bytes as
    /// an integer; the caller stores it into a struct-shaped slot and loads
    /// the aggregate.
    CoerceInt(String),
    /// Win64 aggregate larger than 8 bytes: hidden first pointer parameter,
    /// declared/emitted as `void`, caller loads the value from the slot.
    Sret,
}

impl Default for RetAbi {
    fn default() -> Self {
        RetAbi::Direct
    }
}

/// The C-lowered signature plan for one C function/fn-pointer boundary.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CAbiPlan {
    /// Action per SOURCE parameter (same order/length as `source_params`).
    pub params: Vec<ParamAbi>,
    /// Return action.
    pub ret: RetAbi,
    /// XIOM-value LLVM types of the source parameters.
    pub source_params: Vec<String>,
    /// XIOM-value LLVM return type.
    pub source_ret: String,
    /// LLVM types to use in the `declare`/call for each source parameter
    /// (`%struct.X*` for `IndirectCopy`; the source type otherwise).
    pub c_params: Vec<String>,
    /// LLVM return type to use in the `declare`/call (`void` for `Sret`).
    pub c_ret: String,
    /// The aggregate type carried through the hidden `sret` pointer.
    pub sret_ty: Option<String>,
}

impl CAbiPlan {
    /// True when no parameter/return lowering is required (non-Win64
    /// targets and signatures without aggregates).
    pub fn is_direct(&self) -> bool {
        self.ret == RetAbi::Direct && self.params.iter().all(|p| *p == ParamAbi::Direct)
    }

    /// The full C call parameter type list: the hidden `sret` pointer (when
    /// any) followed by the per-source-parameter C types.
    pub fn call_param_types(&self) -> Vec<String> {
        let mut out = Vec::with_capacity(self.c_params.len() + 1);
        if let Some(ty) = self.sret_ty.as_ref() {
            out.push(format!("{ty}*"));
        }
        out.extend(self.c_params.iter().cloned());
        out
    }
}

/// Is this a C boundary whose lowering must be planned? The confirmed
/// defect target is x86_64 Windows (MS x64 ABI). Do not enable other
/// targets until their aggregate classification is validated.
pub fn is_win64_c_abi(triple: &str) -> bool {
    triple.starts_with("x86_64") && triple.contains("windows")
}

/// System V / AAPCS64 targets that classify aggregates larger than two
/// eightbytes as MEMORY. C passes/returns them through the stack (a `byval`
/// pointer / a hidden sret pointer), never in registers.
pub fn is_sysv_memory_target(triple: &str) -> bool {
    (triple.starts_with("x86_64") || triple.starts_with("aarch64"))
        && !triple.contains("windows")
}

/// A by-value aggregate (`%struct.X`, not a pointer to one) that the C ABI
/// classifies by size.
fn is_by_value_aggregate(llvm_ty: &str) -> bool {
    llvm_ty.starts_with("%struct.") && !llvm_ty.ends_with('*')
}

/// Win64 passes aggregates of exactly these sizes in integer registers.
fn win64_register_size(size: usize) -> bool {
    matches!(size, 1 | 2 | 4 | 8)
}

/// The integer type C coerces a 1/2/4/8-byte aggregate to.
fn win64_coerce_int(size: usize) -> Option<&'static str> {
    match size {
        1 => Some("i8"),
        2 => Some("i16"),
        4 => Some("i32"),
        8 => Some("i64"),
        _ => None,
    }
}

/// Build the plan. `size_of` returns the byte size of an LLVM type.
pub fn build_plan(
    params: &[String],
    ret: &str,
    triple: &str,
    size_of: impl Fn(&str) -> usize,
) -> CAbiPlan {
    let mut plan = CAbiPlan {
        params: vec![ParamAbi::Direct; params.len()],
        ret: RetAbi::Direct,
        source_params: params.to_vec(),
        source_ret: ret.to_string(),
        c_params: params.to_vec(),
        c_ret: ret.to_string(),
        sret_ty: None,
    };
    if !is_win64_c_abi(triple) {
        if is_sysv_memory_target(triple) {
            // >16-byte aggregates are MEMORY class: pass a byval pointer and
            // return through a hidden sret pointer.
            for (i, pty) in params.iter().enumerate() {
                if is_by_value_aggregate(pty) && size_of(pty) > 16 {
                    plan.params[i] = ParamAbi::IndirectCopyByVal;
                    plan.c_params[i] = format!("{pty}*");
                }
            }
            if is_by_value_aggregate(ret) && size_of(ret) > 16 {
                plan.ret = RetAbi::Sret;
                plan.c_ret = "void".to_string();
                plan.sret_ty = Some(ret.to_string());
            }
        }
        return plan;
    }
    for (i, pty) in params.iter().enumerate() {
        if !is_by_value_aggregate(pty) {
            continue;
        }
        let size = size_of(pty);
        if win64_register_size(size) {
            if let Some(ity) = win64_coerce_int(size) {
                plan.params[i] = ParamAbi::CoerceInt(ity.to_string());
                plan.c_params[i] = ity.to_string();
            }
        } else {
            plan.params[i] = ParamAbi::IndirectCopy;
            plan.c_params[i] = format!("{pty}*");
        }
    }
    if is_by_value_aggregate(ret) {
        let size = size_of(ret);
        if win64_register_size(size) {
            if let Some(ity) = win64_coerce_int(size) {
                plan.ret = RetAbi::CoerceInt(ity.to_string());
                plan.c_ret = ity.to_string();
            }
        } else {
            plan.ret = RetAbi::Sret;
            plan.c_ret = "void".to_string();
            plan.sret_ty = Some(ret.to_string());
        }
    }
    plan
}

#[cfg(test)]
mod tests {
    use super::*;

    fn size(ty: &str) -> usize {
        match ty {
            "%struct.Pair2" => 16,
            "%struct.Alloc5" => 40,
            "%struct.Small" => 8,
            "%struct.Tiny" => 4,
            "%struct.Big" => 24,
            _ => 8,
        }
    }

    #[test]
    fn win64_16b_param_is_indirect_and_16b_return_is_sret() {
        let plan = build_plan(
            &["%struct.Pair2".to_string(), "i64".to_string()],
            "%struct.Pair2",
            "x86_64-pc-windows-msvc",
            size,
        );
        assert_eq!(plan.params, vec![ParamAbi::IndirectCopy, ParamAbi::Direct]);
        assert_eq!(plan.ret, RetAbi::Sret);
        assert_eq!(
            plan.c_params,
            vec!["%struct.Pair2*".to_string(), "i64".to_string()]
        );
        assert_eq!(plan.c_ret, "void");
        assert_eq!(plan.sret_ty.as_deref(), Some("%struct.Pair2"));
        assert_eq!(
            plan.call_param_types(),
            vec![
                "%struct.Pair2*".to_string(),
                "%struct.Pair2*".to_string(),
                "i64".to_string()
            ]
        );
    }

    #[test]
    fn win64_40b_param_is_indirect_and_40b_return_is_sret() {
        let plan = build_plan(
            &["%struct.Alloc5".to_string()],
            "%struct.Alloc5",
            "x86_64-pc-windows-msvc",
            size,
        );
        assert_eq!(plan.params, vec![ParamAbi::IndirectCopy]);
        assert_eq!(plan.ret, RetAbi::Sret);
        assert_eq!(plan.c_params, vec!["%struct.Alloc5*".to_string()]);
        assert_eq!(plan.c_ret, "void");
    }

    #[test]
    fn win64_small_aggregates_are_integer_coerced() {
        // C passes 1/2/4/8-byte aggregates as a single integer; LLVM's naked
        // %struct lowering splits them into separate registers, so they must
        // be coerced explicitly (not left Direct).
        let plan = build_plan(
            &["%struct.Small".to_string(), "%struct.Tiny".to_string()],
            "%struct.Small",
            "x86_64-pc-windows-msvc",
            size,
        );
        assert_eq!(
            plan.params,
            vec![
                ParamAbi::CoerceInt("i64".to_string()),
                ParamAbi::CoerceInt("i32".to_string())
            ]
        );
        assert_eq!(plan.ret, RetAbi::CoerceInt("i64".to_string()));
        assert_eq!(plan.c_params, vec!["i64".to_string(), "i32".to_string()]);
        assert_eq!(plan.c_ret, "i64");
        assert!(!plan.is_direct());
    }

    #[test]
    fn win64_padded_aggregate_size_comes_from_the_caller_layout() {
        // `{i32, i8}` occupies 8 bytes after alignment; the caller's size
        // function (not a naive field sum) decides the coercion width.
        let plan = build_plan(
            &["%struct.Padded".to_string()],
            "i64",
            "x86_64-pc-windows-msvc",
            |_| 8,
        );
        assert_eq!(
            plan.params,
            vec![ParamAbi::CoerceInt("i64".to_string())]
        );
    }

    #[test]
    fn win64_odd_sized_aggregates_are_indirect() {
        // 3/5/6/7-byte aggregates are NOT register-class on MS x64.
        let plan = build_plan(
            &["%struct.Odd".to_string()],
            "%struct.Odd",
            "x86_64-pc-windows-msvc",
            |_| 3,
        );
        assert_eq!(plan.params, vec![ParamAbi::IndirectCopy]);
        assert_eq!(plan.ret, RetAbi::Sret);
        assert_eq!(plan.c_params, vec!["%struct.Odd*".to_string()]);
    }

    #[test]
    fn win64_large_return_is_sret() {
        let plan = build_plan(
            &["i64".to_string()],
            "%struct.Big",
            "x86_64-pc-windows-msvc",
            size,
        );
        assert_eq!(plan.ret, RetAbi::Sret);
        assert_eq!(plan.c_ret, "void");
        assert_eq!(plan.sret_ty.as_deref(), Some("%struct.Big"));
    }

    #[test]
    fn sysv_memory_class_aggregates_use_byval_and_sret() {
        // m351: a 40-byte record is MEMORY class on SysV/AAPCS64. The call
        // operand is a byval pointer (C copies it to the outgoing stack) and
        // the return uses the hidden sret pointer.
        for triple in [
            "x86_64-unknown-linux-gnu",
            "x86_64-apple-darwin",
            "aarch64-unknown-linux-gnu",
            "aarch64-apple-darwin",
        ] {
            let plan = build_plan(
                &["%struct.Alloc5".to_string()],
                "%struct.Alloc5",
                triple,
                size,
            );
            assert_eq!(
                plan.params,
                vec![ParamAbi::IndirectCopyByVal],
                "{triple}: 40B param must be byval"
            );
            assert_eq!(plan.c_params, vec!["%struct.Alloc5*".to_string()]);
            assert_eq!(plan.ret, RetAbi::Sret, "{triple}: 40B return is sret");
            assert_eq!(plan.c_ret, "void");
            assert_eq!(plan.sret_ty.as_deref(), Some("%struct.Alloc5"));
        }
    }

    #[test]
    fn sysv_and_aarch64_are_left_on_the_internal_convention() {
        for triple in [
            "x86_64-unknown-linux-gnu",
            "x86_64-apple-darwin",
            "aarch64-unknown-linux-gnu",
            "aarch64-apple-darwin",
            "aarch64-pc-windows-msvc",
        ] {
            let plan = build_plan(
                &["%struct.Pair2".to_string()],
                "%struct.Pair2",
                triple,
                size,
            );
            assert!(
                plan.is_direct(),
                "{triple} must not change the internal convention"
            );
            assert_eq!(plan.c_params, vec!["%struct.Pair2".to_string()]);
            assert_eq!(plan.c_ret, "%struct.Pair2");
            assert_eq!(plan.sret_ty, None);
        }
    }

    #[test]
    fn scalars_and_pointers_are_always_direct() {
        let plan = build_plan(
            &[
                "i64".to_string(),
                "i8*".to_string(),
                "%struct.Pair2*".to_string(),
            ],
            "i64",
            "x86_64-pc-windows-msvc",
            size,
        );
        assert!(plan.is_direct());
        assert_eq!(plan.call_param_types(), plan.source_params);
    }

    #[test]
    fn call_param_types_prepends_the_sret_pointer() {
        let plan = build_plan(
            &["i64".to_string()],
            "%struct.Pair2",
            "x86_64-pc-windows-msvc",
            size,
        );
        assert_eq!(
            plan.call_param_types(),
            vec!["%struct.Pair2*".to_string(), "i64".to_string()]
        );
    }
}
