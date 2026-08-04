#!/usr/bin/env bash
# XIOM Full Test Suite with realtime progress counters.
# Run: ./test_summary.sh
# Shows live test counts as they pass/fail instead of waiting silently.

set -euo pipefail

GREEN='\033[32m'
RED='\033[31m'
YELLOW='\033[33m'
CYAN='\033[36m'
MAGENTA='\033[35m'
DGRAY='\033[90m'
NC='\033[0m'

# ---- helpers ----------------------------------------------------------------
run_suite() {
    local pkg="$1"
    local test_file="$2"
    local label="$3"
    shift 3
    local extra_args=("$@")

    local tmpfile
    tmpfile=$(mktemp /tmp/xiom_test.XXXXXX)

    # Build cargo args
    local cargs=("test" "-p" "$pkg")
    if [ -n "${test_file}" ] && [ "${test_file}" != "_" ]; then
        cargs+=("--test" "$test_file")
    fi
    cargs+=("${extra_args[@]}")

    # Run in background, redirect to temp file
    cargo "${cargs[@]}" > "$tmpfile" 2>&1 &
    local pid=$!

    local passed=0 failed=0 total=0 finished=0
    local label_padded
    printf -v label_padded "%-35s" "${label}"

    # Poll temp file for progress
    while kill -0 "$pid" 2>/dev/null; do
        sleep 0.3
        if [ -f "$tmpfile" ]; then
            # Count ok/failed lines since last check
            local new_passed new_failed
            new_passed=$(grep -c '^test .*\.\.\. ok$' "$tmpfile" 2>/dev/null || echo 0)
            new_failed=$(grep -c '^test .*\.\.\. FAILED$' "$tmpfile" 2>/dev/null || echo 0)
            passed=$new_passed
            failed=$new_failed
            finished=$((passed + failed))

            # Detect total count
            if [ "$total" -eq 0 ]; then
                total=$(grep -m1 'running [0-9]* tests' "$tmpfile" 2>/dev/null | grep -o '[0-9]*' | head -1 || echo 0)
            fi

            # Progress bar
            if [ "$total" -gt 0 ]; then
                local pct=$((finished * 100 / total))
                local bar="" i
                for i in $(seq 1 20); do
                    if [ $((i * 5)) -le $pct ]; then bar+="="; else bar+=" "; fi
                done
                printf "\r  %s[%s] %d/%d (%d pass, %d fail)   " "$label_padded" "$bar" "$finished" "$total" "$passed" "$failed"
            else
                printf "\r  %s%d pass, %d fail   " "$label_padded" "$passed" "$failed"
            fi
        fi
    done
    wait "$pid" 2>/dev/null || true

    # Parse final result
    if [ -f "$tmpfile" ]; then
        local result_line
        result_line=$(grep 'test result:' "$tmpfile" | tail -1)
        passed=$(echo "$result_line" | sed -n 's/.*\([0-9]\+\) passed.*/\1/p')
        failed=$(echo "$result_line" | sed -n 's/.*\([0-9]\+\) failed.*/\1/p')
        local ignored
        ignored=$(echo "$result_line" | sed -n 's/.*\([0-9]\+\) ignored.*/\1/p')
        passed=${passed:-0}
        failed=${failed:-0}
        ignored=${ignored:-0}

        TOTAL_PASSED=$((TOTAL_PASSED + passed))
        TOTAL_FAILED=$((TOTAL_FAILED + failed))
        TOTAL_IGNORED=$((TOTAL_IGNORED + ignored))

        # Clear progress line
        printf "\r  %-35s " "$label"

        if [ "$failed" -gt 0 ]; then
            echo -e "${RED}FAIL ($passed/$((passed + failed)) passed)${NC}"
            FAILED_SUITES+=("$label")
        elif [ "$passed" -gt 0 ]; then
            echo -e "${GREEN} OK  ($passed passed)${NC}"
        else
            echo -e "${RED}CRASH${NC}"
            FAILED_SUITES+=("$label (no result)")
        fi
    else
        echo -e "${RED}CRASH (no output)${NC}"
        FAILED_SUITES+=("$label (no output)")
    fi

    rm -f "$tmpfile"
}

TOTAL_PASSED=0
TOTAL_FAILED=0
TOTAL_IGNORED=0
FAILED_SUITES=()
COMPILER_PASSED=0; COMPILER_FAILED=0; COMPILER_IGNORED=0
TOOLING_PASSED=0;  TOOLING_FAILED=0;  TOOLING_IGNORED=0

echo ""
echo -e "${MAGENTA}XIOM Test Suite (realtime)${NC}"
echo -e "${MAGENTA}===========================${NC}"
echo ""

# ============================================================================
# COMPILER
# ============================================================================
echo -e "${YELLOW}COMPILER ($(date +%H:%M:%S))${NC}"

TOTAL_PASSED=0; TOTAL_FAILED=0; TOTAL_IGNORED=0
for suite in \
    "xiom-codegen e2e_tests                'e2e (2231 tests)'           _" \
    "xiom-codegen feature_regression_tests  'feature-regression (491)'   _" \
    "xiom-codegen stdlib_execution_tests    'stdlib-execution'           _" \
    "xiom-codegen stdlib_tests              'stdlib-compile'             _" \
    "xiom-codegen integration_tests         'integration (128)'          _" \
    "xiom-codegen diff_tests                'diff'                       _" \
    "xiom-codegen full_diff_tests           'full-diff'                  _" \
    "xiom-codegen fuzz_tests                'fuzz'                       _" \
    "xiom-codegen robustness_tests          'robustness'                 _" \
    "xiom-codegen _                         'codegen-unit'               _" \
    "xiom-lexer   _                         'lexer'                      _" \
    "xiom-parser  _                         'parser'                     -- --test-threads=2" \
    "xiom-check   _                         'checker'                    -- --test-threads=2" \
    "xiom-ctfe    _                         'ctfe'                       _" \
    "xiom-graph   _                         'graph'                      _" \
    "xiom-verify  verifier_tests            'verifier'                   _" \
    "xiom-jit     _                         'jit'                        _" \
    "xiom         scripting_tests           'scripting (34)'             _" \
    "xiom         diff_tests                'script-diff (15)'           _"
do
    eval "suite_arr=($suite)"
    pkg="${suite_arr[0]}"
    test="${suite_arr[1]}"
    label="${suite_arr[2]}"
    extra="${suite_arr[3]}"
    if [ "$extra" = "_" ]; then
        run_suite "$pkg" "$test" "$label"
    else
        run_suite "$pkg" "$test" "$label" "$extra"
    fi
done
COMPILER_PASSED=$TOTAL_PASSED
COMPILER_FAILED=$TOTAL_FAILED
COMPILER_IGNORED=$TOTAL_IGNORED
COMPILER_TOTAL=$((COMPILER_PASSED + COMPILER_FAILED))

# ============================================================================
# TOOLING
# ============================================================================
echo ""
echo -e "${YELLOW}TOOLING ($(date +%H:%M:%S))${NC}"

TOTAL_PASSED=0; TOTAL_FAILED=0; TOTAL_IGNORED=0
for suite in \
    "xiom-fmt     _ formatter" \
    "xiom-lsp     _ lsp" \
    "xiom-pkg     _ package-mgr" \
    "xiom-doc     _ doc-gen" \
    "xiom-ffigen  _ ffi-gen" \
    "xiom-mcp     _ mcp-server" \
    "xiom-dbg     _ debugger" \
    "xiom-display _ display"
do
    eval "suite_arr=($suite)"
    pkg="${suite_arr[0]}"
    label="${suite_arr[2]}"
    run_suite "$pkg" "" "$label"
done
TOOLING_PASSED=$TOTAL_PASSED
TOOLING_FAILED=$TOTAL_FAILED
TOOLING_IGNORED=$TOTAL_IGNORED
TOOLING_TOTAL=$((TOOLING_PASSED + TOOLING_FAILED))

# ============================================================================
# TOTALS
# ============================================================================
GRAND_PASSED=$((COMPILER_PASSED + TOOLING_PASSED))
GRAND_FAILED=$((COMPILER_FAILED + TOOLING_FAILED))
GRAND_IGNORED=$((COMPILER_IGNORED + TOOLING_IGNORED))
GRAND_TOTAL=$((COMPILER_TOTAL + TOOLING_TOTAL))

compiler_color=$GREEN; [ "$COMPILER_FAILED" -gt 0 ] && compiler_color=$RED
tooling_color=$GREEN;  [ "$TOOLING_FAILED"  -gt 0 ] && tooling_color=$RED

echo ""
echo "============================================="
echo -e "  COMPILER  ${compiler_color}${COMPILER_PASSED}/${COMPILER_TOTAL}${NC}"
echo -e "  TOOLING   ${tooling_color}${TOOLING_PASSED}/${TOOLING_TOTAL}${NC}"
echo    "  ----------------------------------------"
if [ "$GRAND_FAILED" -eq 0 ] && [ ${#FAILED_SUITES[@]} -eq 0 ]; then
    echo -e "  ${GREEN}ALL $GRAND_TOTAL TESTS PASSED${NC}"
    echo -e "  ${GREEN}TOTAL: $GRAND_PASSED/$GRAND_TOTAL tests passed${NC}"
    if [ "$GRAND_IGNORED" -gt 0 ]; then
        echo -e "  ${YELLOW}($GRAND_IGNORED ignored)${NC}"
    fi
else
    echo -e "  ${RED}$GRAND_PASSED passed, $GRAND_FAILED failed ($GRAND_TOTAL total)${NC}"
    if [ ${#FAILED_SUITES[@]} -gt 0 ]; then
        echo -e "  ${RED}Failures: ${FAILED_SUITES[*]}${NC}"
    fi
fi
echo "============================================="
echo -e "  ${DGRAY}Finished at $(date +%H:%M:%S)${NC}"

if [ "$GRAND_FAILED" -eq 0 ] && [ ${#FAILED_SUITES[@]} -eq 0 ]; then
    echo -e "  ${CYAN}Release tag: $GRAND_PASSED/$GRAND_TOTAL tests${NC}"
fi
