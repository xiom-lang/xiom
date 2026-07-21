@echo off
setlocal enabledelayedexpansion
title XIOM v0.49.8 INSTALLER

:: ============================================================================
:: XIOM Compiler Installer (Windows) — Production Release v0.49.8
:: ============================================================================

type "%~dp0ascii_art.txt"
echo.
echo   Install: %%LOCALAPPDATA%%\xiom (default)
echo   Runtime: Requires CLANG/LLVM on PATH
echo.

:: Choose install directory
set "XIOM_DEFAULT=%LOCALAPPDATA%\xiom"
set /p XIOM_DIR="  [1/5] Install directory [%XIOM_DEFAULT%]: "
if "%XIOM_DIR%"=="" set "XIOM_DIR=%XIOM_DEFAULT%"
set "XIOM_BIN=%XIOM_DIR%\bin"
set "XIOM_MCP=%XIOM_DIR%\mcp"

echo.
echo   Installing to %XIOM_DIR%...
mkdir "%XIOM_DIR%" 2>nul
mkdir "%XIOM_BIN%" 2>nul
mkdir "%XIOM_MCP%" 2>nul

:: Copy binaries
echo   [2/5] Copying binaries...
copy /Y "%~dp0bin\*.exe" "%XIOM_BIN%\" >nul 2>nul
copy /Y "%~dp0bin\xiom-icon.ico" "%XIOM_BIN%\" >nul 2>nul
echo     + xiom, xiom-fmt, xiom-doc, xiom-ffigen, xiom-pkg
echo     + xiom-lsp, xiom-mcp, xiom-dbg, xiom-verify, z3

:: Copy stdlib
echo   [3/5] Copying standard library...
if exist "%~dp0lib\" (
    xcopy /Y /E /Q "%~dp0lib\*" "%XIOM_DIR%\lib\" >nul 2>nul
    echo     + Standard library installed
)

:: Copy runtime
if exist "%~dp0runtime\" (
    xcopy /Y /E /Q "%~dp0runtime\*" "%XIOM_DIR%\runtime\" >nul 2>nul
    echo     + Runtime installed
)

:: Copy MCP configs
echo   [4/5] Setting up MCP configurations...
if exist "%~dp0mcp\" (
    xcopy /Y /E /Q "%~dp0mcp\*" "%XIOM_MCP%\" >nul 2>nul
)
echo     + MCP configs installed to %XIOM_MCP%

:: AI Configuration
echo.
echo   [5/5] AI Configuration (optional - press Enter to skip)
echo   ---------------------------------------------------------
echo   Configure your AI endpoint and API key.
echo   Supported: OpenAI, Anthropic, Ollama, DeepSeek, LiteLLM
echo.
set /p AI_ENDPOINT="  AI Endpoint [skip]: "
if not "%AI_ENDPOINT%"=="" (
    set /p AI_KEY="  AI API Key [skip]: "
    set /p AI_MODEL="  AI Model (gpt-4o, claude, etc.) [gpt-4o]: "
    if "%AI_MODEL%"=="" set AI_MODEL=gpt-4o
    set /p AI_PROVIDER="  AI Provider [openai]: "
    if "%AI_PROVIDER%"=="" set AI_PROVIDER=openai
)

:: Write AI config
if not "%AI_ENDPOINT%"=="" (
    (
        echo # XIOM AI Configuration
        echo XIOM_AI_PROVIDER=%AI_PROVIDER%
        echo XIOM_AI_ENDPOINT=%AI_ENDPOINT%
        echo XIOM_AI_MODEL=%AI_MODEL%
        if not "%AI_KEY%"=="" echo XIOM_AI_API_KEY=%AI_KEY%
        echo XIOM_AI_TIMEOUT=30
        echo XIOM_AI_CACHE_DIR=%XIOM_DIR%
    ) > "%XIOM_DIR%\.xiom_ai_config"
    echo     + AI configuration saved
)

:: Create xiom.bat wrapper
(
echo @echo off
echo REM XIOM Toolchain v0.49.8
echo set "XIOM_BIN=%XIOM_BIN%"
echo set "XIOM_HOME=%XIOM_DIR%"
echo if "%%1"=="" "%%XIOM_BIN%%\xiom.exe" --help ^& goto :eof
echo if "%%1"=="compile" ^( shift ^& "%%XIOM_BIN%%\xiom.exe" %%* ^) ^& goto :eof
echo if "%%1"=="run"     ^( shift ^& "%%XIOM_BIN%%\xiom.exe" --run %%* ^) ^& goto :eof
echo if "%%1"=="build"   ^( shift ^& "%%XIOM_BIN%%\xiom.exe" build %%* ^) ^& goto :eof
echo if "%%1"=="test"    ^( shift ^& "%%XIOM_BIN%%\xiom.exe" --test %%* ^) ^& goto :eof
echo if "%%1"=="fmt"     ^( shift ^& "%%XIOM_BIN%%\xiom-fmt.exe" %%* ^) ^& goto :eof
echo if "%%1"=="doc"     ^( shift ^& "%%XIOM_BIN%%\xiom-doc.exe" %%* ^) ^& goto :eof
echo if "%%1"=="ffigen"  ^( shift ^& "%%XIOM_BIN%%\xiom-ffigen.exe" %%* ^) ^& goto :eof
echo if "%%1"=="pkg"     ^( shift ^& "%%XIOM_BIN%%\xiom-pkg.exe" %%* ^) ^& goto :eof
echo if "%%1"=="lsp"     ^( shift ^& "%%XIOM_BIN%%\xiom-lsp.exe" %%* ^) ^& goto :eof
echo if "%%1"=="mcp"     ^( shift ^& "%%XIOM_BIN%%\xiom-mcp.exe" %%* ^) ^& goto :eof
echo if "%%1"=="verify"  ^( shift ^& "%%XIOM_BIN%%\xiom-verify.exe" %%* ^) ^& goto :eof
echo if "%%1"=="dbg"     ^( shift ^& "%%XIOM_BIN%%\xiom-dbg.exe" %%* ^) ^& goto :eof
echo if "%%1"=="ai"      ^( shift ^& "%%XIOM_BIN%%\xiom.exe" --ai %%* ^) ^& goto :eof
echo REM Unknown — pass through to xiom
echo "%%XIOM_BIN%%\xiom.exe" %%*
) > "%XIOM_BIN%\xiom.bat"

:: PATH Configuration
echo.
echo   PATH Configuration
echo   ------------------
echo   [U] User PATH   (default, no admin)
echo   [S] System PATH (requires admin)
echo   [N] Skip
echo.
set /p PATH_TYPE="  Choose [U/s/N]: "
if /i "%PATH_TYPE%"=="N" goto :skip_path
if "%PATH_TYPE%"=="" set PATH_TYPE=U

set "REG_HIVE=HKCU"
set "REG_KEY=Environment"
if /i "%PATH_TYPE%"=="S" (
    set "REG_HIVE=HKLM"
    set "REG_KEY=SYSTEM\CurrentControlSet\Control\Session Manager\Environment"
    echo   Requesting System PATH...
)

for /f "usebackq tokens=2,*" %%A in (`reg query %REG_HIVE%\%REG_KEY% /v PATH 2^>nul`) do set "CUR_PATH=%%B"
if "%CUR_PATH%"=="" (
    reg add %REG_HIVE%\%REG_KEY% /v PATH /t REG_EXPAND_SZ /d "%XIOM_BIN%" /f >nul 2>nul
) else (
    echo %CUR_PATH% | find /i "%XIOM_BIN%" >nul 2>nul
    if errorlevel 1 (
        reg add %REG_HIVE%\%REG_KEY% /v PATH /t REG_EXPAND_SZ /d "%CUR_PATH%;%XIOM_BIN%" /f >nul 2>nul
    )
)
echo     + Added %XIOM_BIN% to PATH

:: Set XIOM_HOME env
reg add HKCU\Environment /v XIOM_HOME /t REG_SZ /d "%XIOM_DIR%" /f >nul 2>nul
echo     + XIOM_HOME=%XIOM_DIR%

:skip_path

:: Create uninstaller
(
echo @echo off
echo setlocal
echo echo.
echo echo   XIOM Uninstaller v0.49.8
echo echo   This will remove: %XIOM_DIR%
echo echo.
echo set /p CONFIRM="  Continue? [y/N]: "
echo if /i not "%%CONFIRM%%"=="y" exit /b
echo echo.
echo rmdir /s /q "%XIOM_DIR%"
echo reg delete HKCU\Software\Classes\.xi /f ^>nul 2^>nul
echo reg delete HKCU\Software\Classes\XIOM.Source /f ^>nul 2^>nul
echo reg delete HKCU\Environment /v XIOM_HOME /f ^>nul 2^>nul
echo echo   XIOM removed.
echo pause
) > "%XIOM_BIN%\uninstall.bat"

:: Final message
echo.
echo   =========================================
echo     XIOM v0.49.8 INSTALLED SUCCESSFULLY!
echo   =========================================
echo.
echo   Location:   %XIOM_DIR%
echo   Binary:     %XIOM_BIN%\xiom.exe
echo   Wrapper:    %XIOM_BIN%\xiom.bat
echo   MCP Config: %XIOM_MCP%\xiom-mcp-config.json
echo.
echo   Quick Start:
echo     xiom compile hello.xi
echo     xiom --help
echo     xiom ai file.xi
echo.
echo   To uninstall: %XIOM_BIN%\uninstall.bat
echo.
where clang >nul 2>nul
if errorlevel 1 (
    echo   NOTE: clang not found on PATH.
    echo   Install LLVM: winget install LLVM.LLVM
) else (
    for /f "tokens=*" %%C in ('clang --version 2^>^&1 ^| findstr /b "clang"') do echo   Clang: %%C
)
echo.
pause
endlocal