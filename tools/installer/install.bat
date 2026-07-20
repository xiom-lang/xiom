@echo off
setlocal enabledelayedexpansion
title XIOM v0.49.5 INSTALLER

:: ============================================================================
:: XIOM Compiler Installer (Windows)
:: Cross-platform: XIOM_ROOT/bin + XIOM_ROOT/lib + XIOM_ROOT/mcp
:: ============================================================================

echo.
echo.
echo   ^|\  /|  ^|\    /|  ^|\     /|  ^|\  /|  v0.49.5
echo   ^| \/ |  ^| \  / |  ^| \   / |  ^| \/ |  "Phoenix"
echo   ^|    |  ^|  \/  |  ^|  \ /  |  ^|    |  Industrial Compiler
echo   ^|    |  ^|      |  ^|       |  ^|    |  Production Release
echo.
echo   ================================================================
echo     XIOM COMPILER — Lightning-fast systems programming language.
echo     Zero-cost abstractions, contract verification, hot reload.
echo     Built for games, engines, embedded, and high-performance apps.
echo   ================================================================
echo.
echo   Version:  v0.49.5 (881/881 tests, Z3, MCP, LSP, Hot Reload, AI)
echo   Install:  LocalAppData (default) or custom directory
echo   Runtime:  Requires CLANG/LLVM on PATH for native compilation
echo   Docs:     https://xiom-lang.org (coming soon)
echo.

:: ── Choose install directory ──
set "XIOM_DEFAULT=%LOCALAPPDATA%\xiom"
set /p XIOM_DIR="  [1/5] Install directory [%XIOM_DEFAULT%]: "
if "!XIOM_DIR!"=="" set "XIOM_DIR=%XIOM_DEFAULT%"
set "XIOM_BIN=!XIOM_DIR!\bin"
set "XIOM_MCP=!XIOM_DIR!\mcp"

echo.
echo   Installing to !XIOM_DIR!...
mkdir "!XIOM_DIR!" 2>nul
mkdir "!XIOM_BIN!" 2>nul
mkdir "!XIOM_MCP!" 2>nul

:: ── Copy binaries ──
echo   [2/5] Copying binaries...
copy /Y "%~dp0bin\*.exe" "!XIOM_BIN!\" >nul 2>nul
copy /Y "%~dp0bin\xiom-icon.ico" "!XIOM_BIN!\" >nul 2>nul
echo     + xiomc.exe, xiom-fmt.exe, xiom-doc.exe, xiom-ffigen.exe
echo     + xiom-pkg.exe, xiom-lsp.exe, xiom-mcp.exe
echo     + xiom-dbg.exe, xiom-verify.exe, z3.exe

:: ── Copy stdlib ──
echo   [3/5] Copying standard library...
if exist "%~dp0lib\" (
    xcopy /Y /E /Q "%~dp0lib\*" "!XIOM_DIR!\lib\" >nul 2>nul
    echo     + Standard library installed
)

:: ── Copy runtime ──
if exist "%~dp0runtime\" (
    xcopy /Y /E /Q "%~dp0runtime\*" "!XIOM_DIR!\runtime\" >nul 2>nul
    echo     + Runtime installed
)

:: ── Copy MCP configs ──
echo   [4/5] Setting up MCP configurations...
if exist "%~dp0mcp\" (
    xcopy /Y /E /Q "%~dp0mcp\*" "!XIOM_MCP!\" >nul 2>nul
)

:: Create MCP config with actual install path
(
echo {
echo   "xiom": {
echo     "command": "!XIOM_BIN:\=\\!\\xiom-mcp.exe",
echo     "args": [],
echo     "env": {
echo       "XIOM_HOME": "!XIOM_DIR:\=\\!"
echo     }
echo   }
echo }
) > "!XIOM_MCP!\xiom-mcp-config.json"
echo     + MCP configs installed to !XIOM_MCP!

:: ── AI Configuration ──
echo.
echo   [5/5] AI Configuration (optional — press Enter to skip)
echo   ---------------------------------------------------------
echo   XIOM integrates with AI providers for error diagnostics.
echo   Configure your endpoint and API key below, or skip.
echo.
echo   Supported providers: OpenAI, Anthropic, Ollama, DeepSeek, LiteLLM
echo.
set /p AI_ENDPOINT="  AI Endpoint (e.g. https://api.openai.com/v1) [skip]: "
if not "!AI_ENDPOINT!"=="" (
    set /p AI_KEY="  AI API Key [skip]: "
    if "!AI_KEY!"=="" set AI_KEY=
    set /p AI_MODEL="  AI Model (e.g. gpt-4o, claude-sonnet-4-20250514) [gpt-4o]: "
    if "!AI_MODEL!"=="" set AI_MODEL=gpt-4o
    set /p AI_PROVIDER="  AI Provider [openai]: "
    if "!AI_PROVIDER!"=="" set AI_PROVIDER=openai
)

:: Write AI config
if not "!AI_ENDPOINT!"=="" (
    (
        echo # XIOM AI Configuration
        echo XIOM_AI_PROVIDER=!AI_PROVIDER!
        echo XIOM_AI_ENDPOINT=!AI_ENDPOINT!
        echo XIOM_AI_MODEL=!AI_MODEL!
        if not "!AI_KEY!"=="" echo XIOM_AI_API_KEY=!AI_KEY!
        echo XIOM_AI_TIMEOUT=30
        echo XIOM_AI_CACHE_DIR=!XIOM_DIR!

        echo.
        echo # MCP Server — start with: xiom-mcp
        echo # The MCP server provides AI-assisted diagnostics to:
        echo #   - Kilo Code / VS Code:    add to .kilocode/mcp.json
        echo #   - Claude Code:            claude mcp add
        echo #   - Continue.dev:           add to continue config
        echo #   - Any MCP-compatible IDE
    ) > "!XIOM_DIR!\.xiom_ai_config"

    echo     + AI configuration saved (endpoint: !AI_ENDPOINT!)
    echo     + Config file: !XIOM_DIR!\.xiom_ai_config
)

:: ── Create xiom.bat wrapper ──
(
echo @echo off
echo REM XIOM Toolchain v0.49.5
echo set "XIOM_BIN=!XIOM_BIN!"
echo set "XIOM_HOME=!XIOM_DIR!"
echo if "%%1"=="" "%%XIOM_BIN%%\xiomc.exe" --help ^& goto :eof
echo if "%%1"=="compile" ^( shift ^& "%%XIOM_BIN%%\xiomc.exe" %%* ^) ^& goto :eof
echo if "%%1"=="run"     ^( shift ^& "%%XIOM_BIN%%\xiomc.exe" --run %%* ^) ^& goto :eof
echo if "%%1"=="build"   ^( shift ^& "%%XIOM_BIN%%\xiomc.exe" build %%* ^) ^& goto :eof
echo if "%%1"=="test"    ^( shift ^& "%%XIOM_BIN%%\xiomc.exe" --test %%* ^) ^& goto :eof
echo if "%%1"=="fmt"     ^( shift ^& "%%XIOM_BIN%%\xiom-fmt.exe" %%* ^) ^& goto :eof
echo if "%%1"=="doc"     ^( shift ^& "%%XIOM_BIN%%\xiom-doc.exe" %%* ^) ^& goto :eof
echo if "%%1"=="ffigen"  ^( shift ^& "%%XIOM_BIN%%\xiom-ffigen.exe" %%* ^) ^& goto :eof
echo if "%%1"=="pkg"     ^( shift ^& "%%XIOM_BIN%%\xiom-pkg.exe" %%* ^) ^& goto :eof
echo if "%%1"=="lsp"     ^( shift ^& "%%XIOM_BIN%%\xiom-lsp.exe" %%* ^) ^& goto :eof
echo if "%%1"=="mcp"     ^( shift ^& "%%XIOM_BIN%%\xiom-mcp.exe" %%* ^) ^& goto :eof
echo if "%%1"=="verify"  ^( shift ^& "%%XIOM_BIN%%\xiom-verify.exe" %%* ^) ^& goto :eof
echo if "%%1"=="dbg"     ^( shift ^& "%%XIOM_BIN%%\xiom-dbg.exe" %%* ^) ^& goto :eof
echo if "%%1"=="ai"      ^( shift ^& "%%XIOM_BIN%%\xiomc.exe" --ai %%* ^) ^& goto :eof
echo if "%%1"=="graph"   ^( shift ^& "%%XIOM_BIN%%\xiomc.exe" --graph %%* ^) ^& goto :eof
echo REM Unknown subcommand — pass through to xiomc
echo "%%XIOM_BIN%%\xiomc.exe" %%*
) > "!XIOM_BIN!\xiom.bat"

:: ── PATH Configuration ──
echo.
echo   PATH Configuration
echo   ------------------
echo   [U] User PATH   — only your account (default, no admin needed)
echo   [S] System PATH — all users (requires admin)
echo   [N] Skip        — add manually later
echo.
set /p PATH_TYPE="  Choose [U/s/N]: "
if /i "!PATH_TYPE!"=="N" goto :skip_path
if "!PATH_TYPE!"=="" set PATH_TYPE=U

set "REG_HIVE=HKCU"
set "REG_KEY=Environment"
if /i "!PATH_TYPE!"=="S" (
    set "REG_HIVE=HKLM"
    set "REG_KEY=SYSTEM\CurrentControlSet\Control\Session Manager\Environment"
    echo   Requesting System PATH (needs admin)...
)

for /f "usebackq tokens=2,*" %%A in (`reg query !REG_HIVE!\!REG_KEY! /v PATH 2^>nul`) do set "CUR_PATH=%%B"
if "!CUR_PATH!"=="" (
    reg add !REG_HIVE!\!REG_KEY! /v PATH /t REG_EXPAND_SZ /d "!XIOM_BIN!" /f >nul 2>nul
) else (
    echo !CUR_PATH! | find /i "!XIOM_BIN!" >nul 2>nul
    if errorlevel 1 (
        reg add !REG_HIVE!\!REG_KEY! /v PATH /t REG_EXPAND_SZ /d "!CUR_PATH!;!XIOM_BIN!" /f >nul 2>nul
    )
)
echo     + Added !XIOM_BIN! to PATH

:: ── Set XIOM_HOME env ──
reg add HKCU\Environment /v XIOM_HOME /t REG_SZ /d "!XIOM_DIR!" /f >nul 2>nul
echo     + XIOM_HOME=!XIOM_DIR!

:skip_path

:: ── Register .xi file icon ──
echo.
set /p REG_EXT="  Register .xi files with XIOM icon? [y/N]: "
if /i "!REG_EXT!"=="y" (
    reg add HKCU\Software\Classes\.xi /ve /d "XIOM.Source" /f >nul 2>nul
    reg add HKCU\Software\Classes\XIOM.Source /ve /d "XIOM Source File" /f >nul 2>nul
    reg add HKCU\Software\Classes\XIOM.Source\DefaultIcon /ve /d "!XIOM_BIN!\xiom-icon.ico" /f >nul 2>nul
    echo     + .xi files registered
)

:: ── Create uninstaller ──
(
echo @echo off
echo setlocal
echo echo.
echo echo   XIOM Uninstaller v0.49.5
echo echo   =========================
echo echo   This will remove: !XIOM_DIR!
echo echo.
echo set /p CONFIRM="  Continue? [y/N]: "
echo if /i not "%%CONFIRM%%"=="y" exit /b
echo echo.
echo echo   Removing XIOM...
echo rmdir /s /q "!XIOM_DIR!"
echo reg delete HKCU\Software\Classes\.xi /f ^>nul 2^>nul
echo reg delete HKCU\Software\Classes\XIOM.Source /f ^>nul 2^>nul
echo reg delete HKCU\Environment /v XIOM_HOME /f ^>nul 2^>nul
echo echo   XIOM removed. Remove from PATH manually if needed:
echo echo     Rundll32 sysdm.cpl,EditEnvironmentVariables
echo pause
) > "!XIOM_BIN!\uninstall.bat"

:: ── Final message ──
echo.
echo   =========================================
echo     XIOM v0.49.5 INSTALLED SUCCESSFULLY!
echo   =========================================
echo.
echo   Location:   !XIOM_DIR!
echo   Binary:     !XIOM_BIN!\xiomc.exe
echo   Wrapper:    !XIOM_BIN!\xiom.bat
echo   MCP Config: !XIOM_MCP!\xiom-mcp-config.json
echo.
if not "!AI_ENDPOINT!"=="" (
echo   AI Provider: !AI_PROVIDER!  ^(!AI_ENDPOINT!^)
echo   AI Config:   !XIOM_DIR!\.xiom_ai_config
echo.
)
echo   Quick Start:
echo     xiom compile hello.xi
echo     xiom --help
echo     xiom build --graph
echo.
echo   MCP Integration:
echo     Copy !XIOM_MCP!\xiom-mcp-config.json to your agent's MCP config:
echo       - Kilo Code:   .kilocode\mcp.json
echo       - Claude Code:  claude mcp add xiom --config ^<path^>
echo       - Cursor:      .cursor\mcp.json
echo.
echo   AI Diagnostics:
echo     xiom --ai       Compile with AI error analysis
echo     xiom ai         Run AI pipeline on source files
echo.
where clang >nul 2>nul
if errorlevel 1 (
    echo   NOTE: clang not found on PATH.
    echo   XIOM emits LLVM IR; clang compiles it to native .exe.
    echo   Install LLVM: winget install LLVM.LLVM
) else (
    for /f "tokens=*" %%C in ('clang --version 2^>^&1 ^| findstr /b "clang"') do echo   Clang:     %%C
)
echo.
echo   To uninstall: !XIOM_BIN!\uninstall.bat
echo.
pause
endlocal
