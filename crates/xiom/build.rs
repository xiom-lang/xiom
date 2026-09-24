// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

fn main() {
    // FE-16: embed the stdlib pin (`STDLIB_VERSION`) so `xiom doctor` can
    // compare an install's lib/package.xi version against the tag this
    // compiler was built for (catches half-updated installs: new bin/, old
    // lib/). Best effort: builds outside a checkout just skip the check.
    let pin_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("STDLIB_VERSION");
    println!("cargo:rerun-if-changed={}", pin_path.display());
    if let Ok(pin) = std::fs::read_to_string(&pin_path) {
        let pin = pin.trim();
        if !pin.is_empty() {
            println!("cargo:rustc-env=XIOM_STDLIB_PIN={pin}");
        }
    }

    // Embed Windows icon + metadata only when both the HOST and TARGET are Windows.
    // `winres` is only available as a Windows build-dependency, so we gate the
    // entire block behind #[cfg(windows)] to avoid a missing-crate error on
    // Linux/macOS hosts.
    #[cfg(windows)]
    {
        if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
            let mut res = winres::WindowsResource::new();
            res.set_icon("../../resource/img/xiom-icon.ico");
            res.compile().unwrap();
        }
    }
}
