# llama.cpp dynamic backends

This directory is the packaging destination for dynamic backend modules:

- `backends/cpu/`
- `backends/cpu-vulkan/`
- `backends/cpu-cuda/` (optional pack)

They are built with `llama-cpp-2`'s `dynamic-backends` feature
(`GGML_BACKEND_DL=ON`, `GGML_CPU_ALL_VARIANTS=ON`) on the Windows side and copied
here by `scripts/build-windows.ps1`. No binary modules are committed; this file
keeps the layout explicit and the bundle model-free.
