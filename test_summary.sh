#!/usr/bin/env bash
set -euo pipefail

# XIOM Test Suite v3 — Linux/macOS
# Usage: ./test_summary.sh [flags]
#   -fast          Skip E2E/full-diff/fuzz
#   -e2eonly       Just 13 core gate tests  
#   -threads N     Set test threads (default: 8)
#   -logs          Write .testlogs/session_*.txt
#   -cleanbuild    Delete .test_build/ contents
#   -cleanlogs     Delete .testlogs/ contents

FAST=false; E2EONLY=false; THREADS=8; LOGS=false
CLEANBUILD=false; CLEANLOGS=false

for arg in "$@"; do
    case "$arg" in
        -fast) FAST=true ;;
        -e2eonly) E2EONLY=true ;;
        -threads) shift; THREADS="$1" ;;
        -logs) LOGS=true ;;
        -cleanbuild) CLEANBUILD=true ;;
        -cleanlogs) CLEANLOGS=true ;;
        -t) shift; THREADS="$1" ;;
        -f) FAST=true ;;
        -l) LOGS=true ;;
    esac
done

if $CLEANBUILD; then rm -rf .test_build/* 2>/dev/null; echo "Cleaned .test_build"; exit 0; fi
if $CLEANLOGS;  then rm -rf .testlogs/*  2>/dev/null; echo "Cleaned .testlogs";  exit 0; fi

mkdir -p .test_build .testlogs
START_TIME=$(date +%s)
SID=$(date +%Y%m%d_%H%M%S)
LOGPATH=".testlogs/session_$SID.txt"
TOTAL_PASSED=0; TOTAL_FAILED=0; TOTAL_IGNORED=0
GREEN='\033[32m'; RED='\033[31m'; YELLOW='\033[33m'; CYAN='\033[36m'; MAGENTA='\033[35m'; NC='\033[0m'

log() { if $LOGS; then echo "[$(date +%H:%M:%S.%3N)] $1" >> "$LOGPATH"; fi; }

run_test() {
    local pkg="$1"; local test_file="$2"; local label="$3"; local extra="${4:-}"
    local tmp_o; tmp_o=$(mktemp /tmp/xt_o.XXXXXX)
    local tmp_e; tmp_e=$(mktemp /tmp/xt_e.XXXXXX)
    
    local cargs=("test" "-p" "$pkg")
    [[ -n "$test_file" && "$test_file" != "--lib" ]] && cargs+=("--test" "$test_file")
    [[ "$test_file" == "--lib" ]] && cargs+=("--lib")
    [[ -n "$extra" ]] && cargs+=($extra)
    cargs+=("--" "--test-threads=$THREADS")
    
    log "cargo ${cargs[*]}"
    printf "  %-25s " "$label"
    
    local start_ns; start_ns=$(date +%s%N)
    cargo "${cargs[@]}" > "$tmp_o" 2> "$tmp_e" &
    local pid=$!
    local total=0 passed=0 failed=0 tc=false
    
    while kill -0 "$pid" 2>/dev/null; do
        sleep 0.2
        local elapsed; elapsed=$(( ($(date +%s%N) - start_ns) / 1000000000 ))
        [[ $elapsed -gt 1200 ]] && { kill "$pid" 2>/dev/null; printf "\r  %-25s HANG (timeout 20min)\n" "$label"; return; }
        if [[ -f "$tmp_o" ]]; then
            local new_passed new_failed
            new_passed=$(grep -c '^test .*\.\.\. ok$' "$tmp_o" 2>/dev/null || echo 0)
            new_failed=$(grep -c '^test .*\.\.\. FAILED$' "$tmp_o" 2>/dev/null || echo 0)
            [[ $new_passed -ne $passed || $new_failed -ne $failed ]] && {
                passed=$new_passed; failed=$new_failed
                if ! $tc; then
                    total=$(grep -m1 'running [0-9]* tests' "$tmp_o" 2>/dev/null | grep -o '[0-9]*' | head -1 || echo 0)
                    [[ -n "$total" && "$total" -gt 0 ]] && tc=true
                fi
                if $tc && [[ $total -gt 0 ]]; then
                    local fin=$((passed + failed)); local pct=$((fin * 100 / total))
                    local bar=""; for i in $(seq 1 20); do [[ $((i * 5)) -le $pct ]] && bar+="=" || bar+="-"; done
                    printf "\r  [%s] %d/%d (%d ok, %d fail)   " "$bar" "$fin" "$total" "$passed" "$failed"
                fi
            }
        fi
    done
    wait "$pid" 2>/dev/null || true
    
    local elapsed_ms; elapsed_ms=$(( ($(date +%s%N) - start_ns) / 1000000 ))
    local so se p f i crash
    so=$(cat "$tmp_o" 2>/dev/null || echo ""); se=$(cat "$tmp_e" 2>/dev/null || echo "")
    rm -f "$tmp_o" "$tmp_e"
    
    local all; all="$so"$'\n'"$se"
    p=0; f=0; i=0; crash=false
    if echo "$all" | grep -qP 'test result: \w+\.\s*(\d+) passed;\s*(\d+) failed;\s*(\d+) ignored'; then
        local result_line; result_line=$(echo "$all" | grep -P 'test result:' | tail -1)
        p=$(echo "$result_line" | sed -n 's/.*\([0-9]\+\) passed.*/\1/p')
        f=$(echo "$result_line" | sed -n 's/.*\([0-9]\+\) failed.*/\1/p')
        i=$(echo "$result_line" | sed -n 's/.*\([0-9]\+\) ignored.*/\1/p')
        p=${p:-0}; f=${f:-0}; i=${i:-0}
    elif echo "$se" | grep -q 'STATUS_STACK_OVERFLOW\|STATUS_ACCESS_VIOLATION\|STATUS_ILLEGAL'; then
        crash=true
        local cn; cn=$(echo "$se" | grep -o 'STATUS_\w\+' | head -1)
        p=$(echo "$so" | grep -c '\.\.\. ok' || echo 0)
        f=$(echo "$so" | grep -c '\.\.\. FAILED' || echo 0)
    fi
    
    local es; if [[ $elapsed_ms -lt 1000 ]]; then es="${elapsed_ms}ms"; else es="$((elapsed_ms / 1000)).$((elapsed_ms % 1000 / 100))s"; fi
    
    if $crash; then
        echo -e "${MAGENTA}CRASH $cn ($p ok before crash)${NC}"
    elif [[ $f -gt 0 ]]; then
        echo -e "${RED}FAIL ${es} ($p/$((p+f)))${NC}"
    elif [[ $p -gt 0 ]]; then
        echo -e "${GREEN}OK   ${es} ($p)${NC}"
    else
        echo -e "${YELLOW}NONE ${es}${NC}"
    fi
    
    TOTAL_PASSED=$((TOTAL_PASSED + p))
    TOTAL_FAILED=$((TOTAL_FAILED + f))
    TOTAL_IGNORED=$((TOTAL_IGNORED + i))
    log "END $label p=$p f=$f i=$i crash=$crash t=$es"
    
    if [[ $f -gt 0 && "$LOGS" == "true" ]]; then
        local fp; fp=".testlogs/failures_$SID.txt"
        echo "=== $label ($p/$((p+f))) ===" >> "$fp"
        echo "$so" | grep "FAILED" >> "$fp" 2>/dev/null || true
        echo "" >> "$fp"
    fi
}

# ---- Build ----
echo -e "\n${MAGENTA}XIOM Test Suite${NC}"
echo -e "${MAGENTA}===============${NC}"
echo ""
printf "${YELLOW}BUILD (parallel)...${NC} "
log "BUILD START"
BUILD_START=$(date +%s)
cargo test --workspace --no-run > /dev/null 2>&1
BUILD_END=$(date +%s)
if [[ $? -ne 0 ]]; then
    echo -e "${RED}FAILED${NC}"
    exit 1
fi
echo -e "${GREEN}OK ($((BUILD_END - BUILD_START))s)${NC}"
log "BUILD OK"
echo ""

# ---- Run ----
echo -e "${YELLOW}RUN ($THREADS threads) - $(date +%H:%M:%S)${NC}"
log "RUN START threads=$THREADS"

echo -e " ${CYAN}[UNITS]${NC}"
run_test "xiom-lexer"   ""         "lexer"
run_test "xiom-parser"  ""         "parser"
run_test "xiom-check"   ""         "checker"
run_test "xiom-ctfe"    ""         "ctfe"
run_test "xiom-graph"   ""         "graph"
run_test "xiom-codegen" "--lib"    "codegen-unit"
run_test "xiom-verify"  "verifier_tests" "verifier"
run_test "xiom-jit"     ""         "jit"

echo -e " ${CYAN}[TOOLS]${NC}"
run_test "xiom-fmt"     ""         "formatter"
run_test "xiom-lsp"     ""         "lsp"
run_test "xiom-pkg"     ""         "package-mgr"
run_test "xiom-doc"     ""         "doc-gen"
run_test "xiom-ffigen"  ""         "ffi-gen"
run_test "xiom-mcp"     ""         "mcp-server"
run_test "xiom-dbg"     ""         "debugger"
run_test "xiom-display" ""         "display"
run_test "xiom" "scripting_tests"  "scripting"
run_test "xiom" "diff_tests"       "script-diff"

echo -e " ${CYAN}[SUITES]${NC}"
run_test "xiom-codegen" "integration_tests"      "integration"
run_test "xiom-codegen" "diff_tests"             "diff"
$FAST || run_test "xiom-codegen" "full_diff_tests" "full-diff"
$FAST || run_test "xiom-codegen" "fuzz_tests" "fuzz"
run_test "xiom-codegen" "robustness_tests"        "robustness"
run_test "xiom-codegen" "stdlib_execution_tests"  "stdlib-exec"
run_test "xiom-codegen" "stdlib_tests"            "stdlib-compile"
$FAST || run_test "xiom-codegen" "feature_regression_tests" "feature-reg"

if ! $FAST; then
    echo -e " ${CYAN}[E2E]${NC}"
    if $E2EONLY; then
        run_test "xiom-codegen" "e2e_tests" "e2e-gate" "e2e_p0 e2e_p1 e2e_p2 e2e_never_type e2e_asm e2e_spawn_basic e2e_i2"
    else
        run_test "xiom-codegen" "e2e_tests" "e2e (all 2231)"
    fi
else
    echo -e " ${YELLOW}[E2E] SKIPPED${NC}"
fi

# ---- Summary ----
END_TIME=$(date +%s)
TOTAL_TIME=$((END_TIME - START_TIME))
TOTAL=$((TOTAL_PASSED + TOTAL_FAILED))
echo ""
echo -e "${MAGENTA}============================================${NC}"
echo -e "  ${CYAN}Time:   ${TOTAL_TIME}s  |  Passed: $TOTAL_PASSED  |  Failed: $TOTAL_FAILED  |  Total: $TOTAL${NC}"
[[ $TOTAL_IGNORED -gt 0 ]] && echo -e "  ${YELLOW}Ignored: $TOTAL_IGNORED${NC}"
echo -e "${MAGENTA}============================================${NC}"
log "SUMMARY p=$TOTAL_PASSED f=$TOTAL_FAILED i=$TOTAL_IGNORED t=$TOTAL_TIME"
[[ $TOTAL_FAILED -gt 0 ]] && exit 1 || exit 0
