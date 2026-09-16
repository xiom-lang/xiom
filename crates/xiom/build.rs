// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

fn main() {
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
