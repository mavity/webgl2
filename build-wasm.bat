@echo off
REM build-wasm.bat - set MSVC environment and run wasm check/build for this repo
setlocal
REM Call Visual Studio vcvarsall to configure MSVC environment (x64 host)
call "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvarsall.bat" x64
if errorlevel 1 (
  echo Failed to initialize Visual Studio environment. Check that VC is installed.
  endlocal
  exit /b 1
)
REM Ensure wasm target exists (no-op if already installed)
rustup target add wasm32-unknown-unknown 2>nul
REM Run cargo check for wasm target. Pass any extra args through.
cargo check --target wasm32-unknown-unknown %*
set EXIT_CODE=%ERRORLEVEL%
endlocal & exit /b %EXIT_CODE%
