# Windows screenshots (dev VM)

`console-dev-vm.png` is a client-area capture from the QEMU Windows 11 dev VM
using the Microsoft Basic Display Adapter (software-class rendering).

It proves the GPUIX window, the Aurora console screen, and the native napi
bridge start and paint on a GPU-less Windows 11 VM. It is **not** D3D11/12
hardware acceptance and must not be used to claim the W-1 overlay matrix, W-5
performance/accessibility, or any multi-monitor/DPI result.

Hardware captures belong in this folder once the real Windows 11 host runs the
gates. Keep the Linux portfolio in `docs/screenshots/m2/` untouched.
