fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let mut res = winres::WindowsResource::new();
        res.set_icon("../../resource/img/axiom-icon.ico");
        res.compile().unwrap();
    }
}
