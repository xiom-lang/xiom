@echo off
REM XIOM Toolchain Dispatcher v0.20.0

setlocal
set "XIOM_BIN=%LOCALAPPDATA%\xiom\bin"

REM If binaries exist in LOCALAPPDATA, use those
if exist "%XIOM_BIN%\xiomc.exe" goto :check_args

REM Fallback: look in project root (same dir as this bat)
set "XIOM_BIN=%~dp0"
if exist "%XIOM_BIN%\xiomc.exe" goto :check_args

echo XIOM not found. Run install.ps1 first.
exit /b 1

:check_args
if "%1"=="" goto usage
if "%1"=="--help" goto usage

if "%1"=="compile" (
    "%XIOM_BIN%\xiomc.exe" %2 %3 %4 %5 %6 %7 %8 %9
    goto end
)
if "%1"=="fmt" (
    "%XIOM_BIN%\xiom-fmt.exe" %2 %3 %4 %5 %6 %7 %8 %9
    goto end
)
if "%1"=="doc" (
    "%XIOM_BIN%\xiom-doc.exe" %2 %3 %4 %5 %6 %7 %8 %9
    goto end
)
if "%1"=="ffigen" (
    "%XIOM_BIN%\xiom-ffigen.exe" %2 %3 %4 %5 %6 %7 %8 %9
    goto end
)
if "%1"=="pkg" (
    "%XIOM_BIN%\xiom-pkg.exe" %2 %3 %4 %5 %6 %7 %8 %9
    goto end
)
if "%1"=="lsp" (
    "%XIOM_BIN%\xiom-lsp.exe" %2 %3 %4 %5 %6 %7 %8 %9
    goto end
)

REM Fallback: pass through to xiomc
"%XIOM_BIN%\xiomc.exe" %*
goto end

:usage
echo XIOM v0.20.0 -- Toolchain
echo.
echo   xiom compile source.xi     Compile XIOM source
echo   xiom fmt source.xi         Format source
echo   xiom doc source.xi         Generate documentation
echo   xiom ffigen spec.xiom-bind  Generate FFI bindings
echo   xiom pkg --list --root dir Manage packages
echo   xiom lsp                   Language server
echo.
echo   xiomc source.xi            Direct compiler invocation

:end
endlocal
