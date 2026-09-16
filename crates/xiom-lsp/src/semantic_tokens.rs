// XIOM Language Server -- Semantic tokens for syntax highlighting
// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::backend::Backend;

/// Token type indices matching the legend sent in capabilities:
/// 0=keyword, 1=type, 2=function, 3=variable, 4=string, 5=number, 6=comment, 7=operator
pub fn xiom_semantic_token_type(word: &str) -> u32 {
    match word {
        "fn" | "var" | "let" | "if" | "else" | "while" | "for" | "return"
        | "module" | "use" | "pub" | "extern" | "type" | "match" | "spawn"
        | "break" | "continue" | "true" | "false" | "null" | "self" | "Self"
        | "where" | "as" | "in" | "requires" | "ensures" | "invariant" => 0,
        "Int" | "Int8" | "Int16" | "Int32" | "Int64" | "UInt" | "UInt8"
        | "UInt16" | "UInt32" | "UInt64" | "Float32" | "Float64" | "Bool"
        | "Str" | "Vec" | "Map" | "Option" | "Result" | "Unit" | "Ptr" => 1,
        _ => 3,
    }
}

/// Compute semantic tokens for a document. Returns delta-encoded integer array.
pub fn compute_semantic_tokens(backend: &Backend, uri: &str) -> Vec<u32> {
    let docs = backend.documents();
    let text = match docs.get(uri) {
        Some(t) => t.clone(),
        None => return vec![],
    };
    drop(docs);

    let mut lexer = xiom_lexer::Lexer::new(&text);
    let tokens = lexer.tokenize();
    let mut data = Vec::new();

    let mut prev_line: u32 = 0;
    let mut prev_col: u32 = 0;

    for tok in &tokens {
        let line = tok.span.line.max(1) as u32 - 1;
        let col = tok.span.col.max(1) as u32 - 1;
        let len = tok.lexeme.len() as u32;

        if len == 0 { continue; }

        let token_type = match &tok.kind {
            xiom_lexer::TokenKind::Let | xiom_lexer::TokenKind::Var | xiom_lexer::TokenKind::Const
            | xiom_lexer::TokenKind::Fn | xiom_lexer::TokenKind::Return
            | xiom_lexer::TokenKind::Break | xiom_lexer::TokenKind::Continue
            | xiom_lexer::TokenKind::If | xiom_lexer::TokenKind::Elif | xiom_lexer::TokenKind::Else
            | xiom_lexer::TokenKind::Match | xiom_lexer::TokenKind::While | xiom_lexer::TokenKind::For
            | xiom_lexer::TokenKind::In | xiom_lexer::TokenKind::Spawn
            | xiom_lexer::TokenKind::Module | xiom_lexer::TokenKind::Use | xiom_lexer::TokenKind::Pub
            | xiom_lexer::TokenKind::As | xiom_lexer::TokenKind::Type | xiom_lexer::TokenKind::Enum
            | xiom_lexer::TokenKind::Interface | xiom_lexer::TokenKind::Derive
            | xiom_lexer::TokenKind::True | xiom_lexer::TokenKind::False | xiom_lexer::TokenKind::Self_
            | xiom_lexer::TokenKind::None | xiom_lexer::TokenKind::Ok_ | xiom_lexer::TokenKind::Err_
            | xiom_lexer::TokenKind::Unsafe | xiom_lexer::TokenKind::Extern | xiom_lexer::TokenKind::Is
            | xiom_lexer::TokenKind::Some | xiom_lexer::TokenKind::Comptime
            | xiom_lexer::TokenKind::Await => 0,
            xiom_lexer::TokenKind::Ident(s) => xiom_semantic_token_type(s),
            xiom_lexer::TokenKind::Int(_) => 5,
            xiom_lexer::TokenKind::Float(_) => 5,
            xiom_lexer::TokenKind::Str(_) => 4,
            _ => continue,
        };

        let delta_line = line.wrapping_sub(prev_line);
        let delta_col = if delta_line == 0 { col.wrapping_sub(prev_col) } else { col };

        data.push(delta_line);
        data.push(delta_col);
        data.push(len);
        data.push(token_type);
        data.push(0);

        prev_line = line;
        prev_col = col;
    }

    data
}
