#!/bin/bash
# Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
# SPDX-License-Identifier: MIT OR Apache-2.0

set -e
. "$HOME/.cargo/env" 2>/dev/null
export PATH="$HOME/.cargo/bin:$PATH"
# Version stamps are NOT set here: `xiom --version` always reports the
# workspace version (release.yml guards tag == version). Set
# XIOM_RELEASE_TAG / XIOM_RELEASE_STATS only for a real release build.
cd /mnt/e/xiom-lang/xiom
bash package.sh 0.52.9