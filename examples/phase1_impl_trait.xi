// M9.6: impl Trait return types -- E2E test
// Verifies that functions with `impl Trait` return types compile to valid LLVM IR.

interface Display {
    fn show() -> Str;
}

fn get_value() -> impl Display {
    return 42;
}

fn main() -> Int {
    return 0;
}
