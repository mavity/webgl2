@echo off
setlocal enabledelayedexpansion

REM build-wasm-fixed.bat - Enhanced MSVC environment setup for wasm builds

REM Call vcvarsall for x64
call "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvarsall.bat" x64
if errorlevel 1 (
  echo Failed to initialize Visual Studio environment.
  endlocal
  exit /b 1
)

REM Additional SDK path search for Windows 11 SDK
REM Try to set LIB/INCLUDE from Windows 11 SDK if available
for /d %%A in ("C:\Program Files (x86)\Windows Kits\11\Lib\*") do (
  if exist "%%A\um\x64\kernel32.lib" (
    echo Found Windows 11 SDK lib at: %%A\um
    set "SDK_LIB=%%A\um\x64;!SDK_LIB!"
    set "SDK_INCLUDE=%%A\um\..\..\Include\*\um;!SDK_INCLUDE!"
  )
)

REM Try Windows 10 SDK as fallback
for /d %%A in ("C:\Program Files (x86)\Windows Kits\10\Lib\*") do (
  if exist "%%A\um\x64\kernel32.lib" (
    echo Found Windows 10 SDK lib at: %%A\um
    set "SDK_LIB=%%A\um\x64;!SDK_LIB!"
    set "SDK_INCLUDE=%%A\um\..\..\Include\*\um;!SDK_INCLUDE!"
  )
)

REM Prepend SDK paths to environment
if defined SDK_LIB (
  set "LIB=!SDK_LIB!;!LIB!"
  echo Updated LIB paths for SDK
)
if defined SDK_INCLUDE (
  set "INCLUDE=!SDK_INCLUDE!;!INCLUDE!"
  echo Updated INCLUDE paths for SDK
)

REM Ensure wasm target is present
rustup target add wasm32-unknown-unknown 2>nul

REM Run cargo check for wasm
cargo check --target wasm32-unknown-unknown %*
set EXIT_CODE=%ERRORLEVEL%
endlocal & exit /b %EXIT_CODE%
