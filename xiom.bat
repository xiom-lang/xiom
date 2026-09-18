@echo off
REM XIOM Toolchain Dispatcher

setlocal
set "XIOM_BIN=%LOCALAPPDATA%\xiom\bin"

REM If binaries exist in LOCALAPPDATA, use those
if exist "%XIOM_BIN%\xiom.exe" goto :check_args

REM Fallback: look in project root (same dir as this bat)
set "XIOM_BIN=%~dp0"
if exist "%XIOM_BIN%\xiom.exe" goto :check_args

echo XIOM not found. Run install.ps1 first.
exit /b 1

:check_args
if "%1"=="" goto :usage
if "%1"=="--help" goto :usage

if "%1"=="compile" (
    "%XIOM_BIN%\xiom.exe" %2 %3 %4 %5 %6 %7 %8 %9
    goto :end
)
if "%1"=="run" (
    "%XIOM_BIN%\xiom.exe" --run %2 %3 %4 %5 %6 %7 %8 %9
    goto :end
)
if "%1"=="build" (
    "%XIOM_BIN%\xiom.exe" build %2 %3 %4 %5 %6 %7 %8 %9
    goto :end
)
if "%1"=="test" (
    "%XIOM_BIN%\xiom.exe" --test %2 %3 %4 %5 %6 %7 %8 %9
    goto :end
)
if "%1"=="fmt" (
    "%XIOM_BIN%\xiom-fmt.exe" %2 %3 %4 %5 %6 %7 %8 %9
    goto :end
)
if "%1"=="doc" (
    "%XIOM_BIN%\xiom-doc.exe" %2 %3 %4 %5 %6 %7 %8 %9
    goto :end
)
if "%1"=="ffigen" (
    "%XIOM_BIN%\xiom-ffigen.exe" %2 %3 %4 %5 %6 %7 %8 %9
    goto :end
)
if "%1"=="pkg" (
    "%XIOM_BIN%\xiom-pkg.exe" %2 %3 %4 %5 %6 %7 %8 %9
    goto :end
)
if "%1"=="lsp" (
    "%XIOM_BIN%\xiom-lsp.exe" %2 %3 %4 %5 %6 %7 %8 %9
    goto :end
)
if "%1"=="mcp" (
    "%XIOM_BIN%\xiom-mcp.exe" %2 %3 %4 %5 %6 %7 %8 %9
    goto :end
)
if "%1"=="dbg" (
    "%XIOM_BIN%\xiom-dbg.exe" %2 %3 %4 %5 %6 %7 %8 %9
    goto :end
)
if "%1"=="verify" (
    "%XIOM_BIN%\xiom-verify.exe" %2 %3 %4 %5 %6 %7 %8 %9
    goto :end
)
if "%1"=="ai" (
    "%XIOM_BIN%\xiom.exe" --ai %2 %3 %4 %5 %6 %7 %8 %9
    goto :end
)
if "%1"=="graph" (
    "%XIOM_BIN%\xiom.exe" --graph %2 %3 %4 %5 %6 %7 %8 %9
    goto :end
)
if "%1"=="doctor" (
    "%XIOM_BIN%\xiom.exe" --doctor %2 %3 %4 %5 %6 %7 %8 %9
    goto :end
)

REM Fallback: pass through to xiom
"%XIOM_BIN%\xiom.exe" %*
goto :end

:usage
echo XIOM -- Toolchain
echo.
echo   xiom compile source.xi      Compile XIOM source
echo   xiom run source.xi          Compile and run
echo   xiom build                  Build project
echo   xiom test                   Run tests
echo   xiom fmt source.xi          Format source
echo   xiom doc source.xi          Generate documentation
echo   xiom ffigen spec.xiom-bind  Generate FFI bindings
echo   xiom pkg --list --root dir  Manage packages
echo   xiom lsp                    Language server
echo   xiom mcp                    MCP server (AI diagnostics)
echo   xiom dbg                    Debugger
echo   xiom verify file.xi         Contract verification
echo   xiom ai source.xi           AI-assisted diagnostics
echo   xiom graph file.xi          Dependency graph
echo   xiom doctor                 Environment diagnostics
echo.
echo   xiom source.xi             Direct compiler invocation

:end
endlocal
