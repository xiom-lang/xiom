#!/bin/bash
# Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
# SPDX-License-Identifier: MIT OR Apache-2.0

set -e
. "$HOME/.cargo/env" 2>/dev/null
export PATH="$HOME/.cargo/bin:$PATH"
export XIOM_RELEASE_TAG="Production Hardening"
export XIOM_RELEASE_STATS="1067/1067 tests, M19 complete"
cd /mnt/e/Projects/AXIOM
bash package.sh 0.52.9