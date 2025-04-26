@echo off
set RUST_TOOLCHAIN_PATH=%USERPROFILE%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\bin

set DEPS_PATH=%CD%\target\debug\deps

set PATH=%RUST_TOOLCHAIN_PATH%;%DEPS_PATH%;%PATH%

start "" "C:\Program Files\NVIDIA Corporation\Nsight Graphics 2025.1.1\host\windows-desktop-nomad-x64\ngfx-ui.exe"