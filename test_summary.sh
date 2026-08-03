#!/usr/bin/env bash
# XIOM Full Test Suite — compiler + tooling. Single-line release tag.
# Run: ./test_summary.sh
# Output last line: "Release tag: NNN/NNN tests" (ready for release tagging)

set -euo pipefail

GREEN='\033[32m'
RED='\033[31m'
YELLOW='\033[33m'
CYAN='\033[36m'
MAGENTA='\033[35m'
NC='\033[0m'

# ---- helpers ----------------------------------------------------------------
run_suite() {
    local pkg="$1"
    local test_file="$2"
    local label="$3"
    shift 3
    local extra_args=("$@")

    printf "  %-22s " "${label}"

    local output
    if [ -z "${test_file}" ] || [ "${test_file}" = "_" ]; then
        output=$(cargo test -p "$pkg" "${extra_args[@]}" 2>&1) || true
    else
        output=$(cargo test -p "$pkg" --test "$test_file" "${extra_args[@]}" 2>&1) || true
    fi

    if echo "$output" | grep -q 'test result:'; then
        local result_line
        result_line=$(echo "$output" | grep 'test result:')
        local passed failed ignored
        passed=$(echo "$result_line" | sed -n 's/.*\([0-9]\+\) passed.*/\1/p')
        failed=$(echo "$result_line" | sed -n 's/.*\([0-9]\+\) failed.*/\1/p')
        ignored=$(echo "$result_line" | sed -n 's/.*\([0-9]\+\) ignored.*/\1/p')
        passed=${passed:-0}
        failed=${failed:-0}
        ignored=${ignored:-0}

        TOTAL_PASSED=$((TOTAL_PASSED + passed))
        TOTAL_FAILED=$((TOTAL_FAILED + failed))
        TOTAL_IGNORED=$((TOTAL_IGNORED + ignored))

        if [ "$failed" -gt 0 ]; then
            echo -e "${RED}FAIL ($passed/$((passed + failed)))${NC}"
            FAILED_SUITES+=("$label")
        else
            echo -e "${GREEN}OK ($passed/$((passed + failed)))${NC}"
        fi
    else
        echo -e "${RED}CRASH${NC}"
        FAILED_SUITES+=("$label (no result)")
    fi
}

TOTAL_PASSED=0
TOTAL_FAILED=0
TOTAL_IGNORED=0
FAILED_SUITES=()
COMPILER_PASSED=0; COMPILER_FAILED=0; COMPILER_IGNORED=0
TOOLING_PASSED=0;  TOOLING_FAILED=0;  TOOLING_IGNORED=0

echo ""
echo -e "${MAGENTA}XIOM Test Suite${NC}"
echo -e "${MAGENTA}==============${NC}"
echo ""

# ============================================================================
# COMPILER
# ============================================================================
echo -e "${YELLOW}COMPILER${NC}"

TOTAL_PASSED=0; TOTAL_FAILED=0; TOTAL_IGNORED=0
for suite in \
    "xiom-codegen e2e_tests                e2e                _" \
    "xiom-codegen feature_regression_tests  feature-regression _" \
    "xiom-codegen stdlib_execution_tests    stdlib-execution   _" \
    "xiom-codegen stdlib_tests              stdlib-compile     _" \
    "xiom-codegen integration_tests         integration        _" \
    "xiom-codegen diff_tests                diff               _" \
    "xiom-codegen full_diff_tests           full-diff          _" \
    "xiom-codegen fuzz_tests                fuzz               _" \
    "xiom-codegen robustness_tests          robustness         _" \
    "xiom-codegen _                         codegen-unit       _" \
    "xiom-lexer   _                         lexer              _" \
    "xiom-parser  _                         parser             -- --test-threads=2" \
    "xiom-check   _                         checker            -- --test-threads=2" \
    "xiom-ctfe    _                         ctfe               _" \
    "xiom-graph   _                         graph              _" \
    "xiom-verify  verifier_tests            verifier           _" \
    "xiom-jit     _                         jit                _" \
    "xiom         scripting_tests           scripting          _" \
    "xiom         diff_tests                script-diff        _"
do
    read -r pkg test label extra <<< "$suite"
    if [ "$extra" = "_" ]; then
        run_suite "$pkg" "$test" "$label"
    else
        run_suite "$pkg" "$test" "$label" "$extra" "--test-threads=2"
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
echo -e "${YELLOW}TOOLING${NC}"

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
    read -r pkg _ label <<< "$suite"
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
    echo -e "  ${RED}Failures: ${FAILED_SUITES[*]}${NC}"
fi
echo "============================================="

if [ "$GRAND_FAILED" -eq 0 ] && [ ${#FAILED_SUITES[@]} -eq 0 ]; then
    echo -e "  ${CYAN}Release tag: $GRAND_PASSED/$GRAND_TOTAL tests${NC}"
fi
