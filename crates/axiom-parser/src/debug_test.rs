use axiom_lexer::Lexer;

fn main() {
    let source = std::fs::read_to_string(r"E:\Projects\AXIOM\test5.ax").unwrap();
    eprintln!("=== Source ===");
    eprintln!("{}", source);
    eprintln!("=== Tokens ===");
    let tokens = Lexer::new(&source).tokenize();
    for (i, t) in tokens.iter().enumerate() {
        eprintln!("  [{}] {:?}  lexeme={:?}  span={:?}", i, t.kind, t.lexeme, t.span);
    }
    eprintln!("=== Parse ===");
    let result = axiom_parser::Parser::new(tokens).parse_program();
    match result {
        Ok(_) => eprintln!("OK"),
        Err(e) => eprintln!("ERROR: {:?}", e),
    }
}
