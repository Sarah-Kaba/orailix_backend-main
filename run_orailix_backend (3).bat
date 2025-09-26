@echo off
rustc --version >nul 2>&1
if %errorlevel% neq 0 (
    echo Rust is not installed. Please install Rust from https://rustup.rs/
    pause
    exit /b 1
)
cargo build
if %errorlevel% neq 0 (
    echo Build failed.
    pause
    exit /b 1
)
target\debug\orailix_backend.exe --ip 127.0.0.1:8080 --origin o
pause
