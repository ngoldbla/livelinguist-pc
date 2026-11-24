# CMake toolchain file for *-pc-windows-msvc builds (llama-cpp-sys-2).
# With policy CMP0091 (CMake >= 3.15) the VS generator sets the C runtime from
# CMAKE_MSVC_RUNTIME_LIBRARY and ignores /MT in CMAKE_C_FLAGS, so the static
# runtime has to be requested here. See ADR 0001, addendum 2026-11-24.
set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreaded$<$<CONFIG:Debug>:Debug>")
