// XIOM -- Structural Type Names (Stage 2c)
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

//! Canonical structural identity for XIOM's string-encoded type names.
//!
//! Lives in `xiom-ast` so every pass (checker, codegen, driver, LSP) shares
//! ONE bracket-aware parser/renderer. XIOM encodes compound types as strings
//! (`"Vec[Option[Int]]"`, `"Result[Int, Str]"`, `"*T"`, `"Tuple__A__B"`);
//! before Stage 2c every consumer split those strings by hand
//! (`split_once('[')`, `contains('[')`, `ends_with(".Name")`), which is where
//! the bare-name collisions and the container ABI bridges came from.
//!
//! - [`parse_type_shape`] turns a name into a [`TypeShape`] tree. It never
//!   panics and never loses information: malformed text falls back to
//!   [`TypeShape::Opaque`] so comparisons stay text-exact.
//! - [`parse_type_arg_list`] splits a top-level argument list at bracket
//!   depth 0 (the old ad-hoc split).
//! - [`canonical_type_name`] renders the canonical spelling
//!   (`"Result[Int,Str]"` -> `"Result[Int, Str]"`), which is what the
//!   checker's interned `TypeArena` interns.
//!
//! No other module should parse type strings by hand; classify through
//! [`TypeShape`] instead.

use std::fmt;

/// A parsed structural view of one type name.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeShape {
    /// A named/compound type: `Int`, `Foo`, `Vec[Int]`, `Map[Str, Int]`.
    Named { base: String, args: Vec<TypeShape> },
    /// A raw pointer (`*T`), exactly one level as encoded by
    /// `CheckedType::from_ast_type`.
    Pointer(Box<TypeShape>),
    /// A single uppercase generic parameter (`T`, `K`, `V`).
    Generic(String),
    /// The unit payload `()`.
    Unit,
    /// The wildcard type `_`.
    Wildcard,
    /// Unparseable text -- compared verbatim, never treated as structural.
    Opaque(String),
}

impl TypeShape {
    /// Base name without arguments (`"Vec[Int]"` -> `"Vec"`).
    pub fn base(&self) -> &str {
        match self {
            TypeShape::Named { base, .. } => base,
            TypeShape::Opaque(text) => text,
            _ => "",
        }
    }

    /// Structural arguments (empty for non-container shapes).
    pub fn args(&self) -> &[TypeShape] {
        match self {
            TypeShape::Named { args, .. } => args,
            _ => &[],
        }
    }

    /// True when the shape carries at least one argument (`Vec[Int]`, not `Vec`).
    pub fn is_container(&self) -> bool {
        matches!(self, TypeShape::Named { args, .. } if !args.is_empty())
    }

    /// True for the empty name the checker uses to erase bare payloads.
    pub fn is_empty(&self) -> bool {
        matches!(self, TypeShape::Named { base, args } if base.is_empty() && args.is_empty())
    }

    /// True when the shape is the wildcard `_`.
    pub fn is_wildcard(&self) -> bool {
        matches!(self, TypeShape::Wildcard)
    }

    /// True when the shape is `()`.
    pub fn is_unit(&self) -> bool {
        matches!(self, TypeShape::Unit)
    }

    /// True for a generic parameter, including one pointer level (`T`, `*T`)
    /// -- the historical `strip_prefix('*')` rule.
    pub fn is_generic_param(&self) -> bool {
        match self {
            TypeShape::Generic(_) => true,
            TypeShape::Pointer(inner) => matches!(inner.as_ref(), TypeShape::Generic(_)),
            _ => false,
        }
    }

    /// True for pointer-like shapes: `*T` or the legacy bare `Ptr`.
    pub fn is_pointer_like(&self) -> bool {
        match self {
            TypeShape::Pointer(_) => true,
            TypeShape::Named { base, .. } => base == "Ptr" || base.starts_with('*'),
            _ => false,
        }
    }

    /// True for tuple-encoded names (`Tuple__A__B`).
    pub fn is_tuple_like(&self) -> bool {
        self.base().starts_with("Tuple")
    }

    /// The scalar text used for `CheckedType::from_str` classification:
    /// the base for named shapes, the canonical rendering for pointer/opaque
    /// shapes. Only called after container/wildcard/generic/tuple checks.
    pub fn scalar_name(&self) -> String {
        match self {
            TypeShape::Opaque(text) => text.clone(),
            TypeShape::Named { base, .. } => base.clone(),
            other => other.to_string(),
        }
    }
}

impl fmt::Display for TypeShape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeShape::Named { base, args } if args.is_empty() => write!(f, "{base}"),
            TypeShape::Named { base, args } => {
                write!(f, "{base}[")?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{arg}")?;
                }
                write!(f, "]")
            }
            TypeShape::Pointer(inner) => write!(f, "*{inner}"),
            TypeShape::Generic(name) => write!(f, "{name}"),
            TypeShape::Unit => write!(f, "()"),
            TypeShape::Wildcard => write!(f, "_"),
            TypeShape::Opaque(text) => write!(f, "{text}"),
        }
    }
}

/// Parse one type name into its structural shape.
///
/// Whitespace around the name, the base, and every argument is ignored.
/// Malformed bracket text (unclosed or trailing garbage) returns
/// [`TypeShape::Opaque`] with the trimmed original so equality stays exact.
pub fn parse_type_shape(name: &str) -> TypeShape {
    let text = name.trim();
    if text.is_empty() {
        return TypeShape::Named { base: String::new(), args: Vec::new() };
    }
    if text == "_" {
        return TypeShape::Wildcard;
    }
    if text == "()" {
        return TypeShape::Unit;
    }
    if let Some(rest) = text.strip_prefix('*') {
        if rest.trim().is_empty() {
            return TypeShape::Named { base: "*".to_string(), args: Vec::new() };
        }
        return TypeShape::Pointer(Box::new(parse_type_shape(rest)));
    }
    if let Some(open) = text.find('[') {
        let base = text[..open].trim().to_string();
        return match parse_bracketed_args(&text[open..]) {
            Some(args) => TypeShape::Named { base, args },
            None => TypeShape::Opaque(text.to_string()),
        };
    }
    if is_generic_param_name(text) {
        return TypeShape::Generic(text.to_string());
    }
    TypeShape::Named { base: text.to_string(), args: Vec::new() }
}

/// Split a top-level comma-separated argument list at bracket depth 0.
/// The empty string yields an empty list; whitespace is trimmed.
pub fn parse_type_arg_list(list: &str) -> Vec<TypeShape> {
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut current = String::new();
    for ch in list.chars() {
        match ch {
            '[' => { depth += 1; current.push(ch); }
            ']' => { depth -= 1; current.push(ch); }
            ',' if depth == 0 => {
                let arg = current.trim();
                if !arg.is_empty() {
                    out.push(parse_type_shape(arg));
                }
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    let arg = current.trim();
    if !arg.is_empty() {
        out.push(parse_type_shape(arg));
    }
    out
}

/// Canonical spelling of a type name: whitespace-normalized, arguments
/// separated by exactly ", " (`"Result[Int,Str]"` -> `"Result[Int, Str]"`).
pub fn canonical_type_name(name: &str) -> String {
    parse_type_shape(name).to_string()
}

/// Container decomposition: `"Option[Result[Int, Str]]"` ->
/// `("Option", ["Result[Int, Str]"])`. Returns None for non-containers,
/// bare names, pointers, wildcards, units, malformed ([`TypeShape::Opaque`])
/// text and tuples. Argument renderings are CANONICAL (structural Display),
/// which is what makes `"Map[Str,Vec[Int]]"` and `"Map[Str, Vec[Int]]"`
/// identical to every consumer.
pub fn container_parts(name: &str) -> Option<(String, Vec<String>)> {
    match parse_type_shape(name) {
        TypeShape::Named { base, args } if !args.is_empty() => {
            Some((base, args.iter().map(|a| a.to_string()).collect()))
        }
        _ => None,
    }
}

/// True when `name` is a container whose BASE is `base` (`"Vec[Int]"` for
/// `base = "Vec"`). Bare names and malformed text are false; use
/// [`TypeShape::base`] when bare forms must also match.
pub fn is_container_base(name: &str, base: &str) -> bool {
    matches!(
        parse_type_shape(name),
        TypeShape::Named { base: b, args } if !args.is_empty() && b == base
    )
}

/// True for the `Result`/`Option` containers (bare or with arguments) plus
/// the checker's `_` placeholder -- the classifier behind the `?` operator.
pub fn is_result_or_option(name: &str) -> bool {
    if name.trim() == "_" {
        return true;
    }
    matches!(
        parse_type_shape(name),
        TypeShape::Named { base, .. } if base == "Result" || base == "Option"
    )
}

/// Split a tuple type name into its element type renderings, accepting BOTH
/// spellings the checker produces:
/// - the parenthesized form `"(Int, Str)"` (depth-aware comma split, so
///   nested brackets/parens survive);
/// - the legacy registered form `"Tuple__Int__Str"` (double-underscore
///   separator, first segment is the `Tuple` marker).
///
/// Returns None for non-tuples, malformed text and single-element legacy
/// text (the historical `< 2` parts guard). Elements are canonical.
pub fn tuple_elem_names(name: &str) -> Option<Vec<String>> {
    let text = name.trim();
    if let Some(inner) = text.strip_prefix('(').and_then(|r| r.strip_suffix(')')) {
        let mut out = Vec::new();
        let mut depth = 0i32;
        let mut current = String::new();
        for ch in inner.chars() {
            match ch {
                '[' | '(' => { depth += 1; current.push(ch); }
                ']' | ')' => { depth -= 1; current.push(ch); }
                ',' if depth == 0 => {
                    let arg = current.trim();
                    if !arg.is_empty() {
                        out.push(parse_type_shape(arg).to_string());
                    }
                    current.clear();
                }
                _ => current.push(ch),
            }
        }
        let arg = current.trim();
        if !arg.is_empty() {
            out.push(parse_type_shape(arg).to_string());
        }
        if out.is_empty() {
            return None;
        }
        return Some(out);
    }
    let rest = text.strip_prefix("Tuple__")?;
    let parts: Vec<String> = rest.split("__")
        .map(|p| parse_type_shape(p).to_string())
        .collect();
    if parts.len() < 2 {
        return None;
    }
    Some(parts)
}

/// True when the text is a single uppercase generic parameter name.
/// Shared with the checker's scalar rules (T, K, V -- not Ty).
pub fn is_generic_param_name(text: &str) -> bool {
    text.len() == 1 && text.chars().next().map_or(false, |c| c.is_ascii_uppercase())
}

/// Parse `[A, B, ...]` starting at the opening bracket. Returns None when the
/// bracket is never closed or trailing text follows the closing bracket.
fn parse_bracketed_args(text: &str) -> Option<Vec<TypeShape>> {
    debug_assert!(text.starts_with('['));
    let mut depth = 0i32;
    for (i, ch) in text.char_indices() {
        match ch {
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    let inner = &text[1..i];
                    let rest = text[i + 1..].trim();
                    if !rest.is_empty() {
                        return None;
                    }
                    return Some(parse_type_arg_list(inner));
                }
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nested_containers() {
        let shape = parse_type_shape("Vec[Option[Int]]");
        assert_eq!(shape.base(), "Vec");
        assert!(shape.is_container());
        assert_eq!(shape.args().len(), 1);
        let inner = &shape.args()[0];
        assert_eq!(inner.base(), "Option");
        assert_eq!(inner.args()[0].base(), "Int");
        assert!(!inner.args()[0].is_container());
    }

    #[test]
    fn parses_multi_arg_containers_with_whitespace() {
        for text in ["Result[Int, Str]", "Result[Int,Str]", " Result[ Int , Str ] "] {
            let shape = parse_type_shape(text);
            assert_eq!(shape.base(), "Result");
            assert_eq!(shape.args().len(), 2);
            assert_eq!(shape.args()[0].base(), "Int");
            assert_eq!(shape.args()[1].base(), "Str");
        }
    }

    #[test]
    fn canonicalizes_spacing() {
        assert_eq!(canonical_type_name("Result[Int,Str]"), "Result[Int, Str]");
        assert_eq!(canonical_type_name("Vec[Vec[Int]]"), "Vec[Vec[Int]]");
        assert_eq!(canonical_type_name("Map[Str, Vec[Int]]"), "Map[Str, Vec[Int]]");
        assert_eq!(canonical_type_name("Int"), "Int");
    }

    #[test]
    fn classifies_wildcard_unit_pointer_generic() {
        assert!(parse_type_shape("_").is_wildcard());
        assert!(parse_type_shape("()").is_unit());
        assert!(parse_type_shape("T").is_generic_param());
        assert!(parse_type_shape("*T").is_generic_param());
        assert!(parse_type_shape("*T").is_pointer_like());
        assert!(parse_type_shape("*Int").is_pointer_like());
        assert!(!parse_type_shape("*Int").is_generic_param());
        assert!(parse_type_shape("Ptr").is_pointer_like());
        assert!(parse_type_shape("Tuple__Int__Str").is_tuple_like());
        assert!(parse_type_shape("").is_empty());
    }

    #[test]
    fn malformed_text_is_opaque_never_lost() {
        let shape = parse_type_shape("Vec[Int");
        assert_eq!(shape, TypeShape::Opaque("Vec[Int".to_string()));
        assert_eq!(shape.scalar_name(), "Vec[Int");
        let trailing = parse_type_shape("Vec[Int]garbage");
        assert_eq!(trailing, TypeShape::Opaque("Vec[Int]garbage".to_string()));
        // Canonicalization of opaque text is identity.
        assert_eq!(canonical_type_name("Vec[Int"), "Vec[Int");
    }

    #[test]
    fn arg_list_split_is_bracket_aware() {
        let args = parse_type_arg_list("Vec[Int], Str");
        assert_eq!(args.len(), 2);
        assert_eq!(args[0].base(), "Vec");
        assert_eq!(args[0].args()[0].base(), "Int");
        assert_eq!(args[1].base(), "Str");
        assert!(parse_type_arg_list("").is_empty());
        assert_eq!(parse_type_arg_list("Int").len(), 1);
    }

    #[test]
    fn container_parts_decomposes_and_canonicalizes() {
        let (base, args) = container_parts("Option[Result[Int,Str]]").expect("container");
        assert_eq!(base, "Option");
        assert_eq!(args, vec!["Result[Int, Str]".to_string()]);
        let (base, args) = container_parts(" Map[ Str ,Vec[Int]] ").expect("container");
        assert_eq!(base, "Map");
        assert_eq!(args, vec!["Str".to_string(), "Vec[Int]".to_string()]);
        assert!(container_parts("Int").is_none());
        assert!(container_parts("Vec").is_none());
        assert!(container_parts("*T").is_none());
        assert!(container_parts("Vec[Int").is_none());
        assert!(container_parts("Tuple__Int__Str").is_none());
        assert!(is_container_base("Vec[Int]", "Vec"));
        assert!(!is_container_base("Vec", "Vec"));
        assert!(!is_container_base("Vec[Int]", "Map"));
    }

    #[test]
    fn tuple_elem_names_both_spellings() {
        assert_eq!(
            tuple_elem_names("(Int, Str)").unwrap(),
            vec!["Int".to_string(), "Str".to_string()]
        );
        assert_eq!(
            tuple_elem_names("Tuple__Int__Str").unwrap(),
            vec!["Int".to_string(), "Str".to_string()]
        );
        // Depth-aware: nested tuple parens survive the split.
        assert_eq!(
            tuple_elem_names("(K, (A, B))").unwrap(),
            vec!["K".to_string(), "(A, B)".to_string()]
        );
        assert!(tuple_elem_names("Int").is_none());
        assert!(tuple_elem_names("Tuple__Int").is_none());
        assert!(tuple_elem_names("()").is_none());
    }

    #[test]
    fn display_round_trips_parsed_shapes() {
        let names = [
            "Int", "Str", "_", "()", "T", "*Int", "Ptr",
            "Vec[Int]", "Option[Vec[Str]]", "Result[Int, Str]",
            "Map[Str, Vec[Int]]", "Tuple__Int__Str",
        ];
        for name in names {
            assert_eq!(canonical_type_name(name), name, "round trip {name}");
        }
    }
}
