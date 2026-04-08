@echo off
REM OpenCoWork Rust — Windows Quick Installer
REM
REM This batch file runs the PowerShell installer.
REM Right-click → Run as Administrator for best results.
REM
REM Usage:
REM   install.bat              Full install
REM   install.bat --cuda       With GPU acceleration
REM   install.bat --help       Show options

echo.
echo   Detecting PowerShell...

where powershell >nul 2>&1
if %errorlevel% neq 0 (
    echo   ERROR: PowerShell not found. Windows 10+ required.
    pause
    exit /b 1
)

echo   Running installer...
echo.

set ARGS=%*

REM Convert common flags
if "%1"=="--cuda" set ARGS=-Cuda
if "%1"=="--help" set ARGS=-Help
if "%1"=="--skip-models" set ARGS=-SkipModels
if "%1"=="--skip-rust" set ARGS=-SkipRust
if "%1"=="--skip-llama" set ARGS=-SkipLlama

powershell -ExecutionPolicy Bypass -File "%~dp0install.ps1" %ARGS%

if %errorlevel% equ 0 (
    echo.
    echo   Installation complete!
    echo   Check your Desktop for the OpenCoWork shortcut.
    echo.
)

pause
