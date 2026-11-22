#!/usr/bin/env node
// Hardware probe contract: the renderer and accelerator status are measured,
// typed, and content-free. Runs headless (DXGI/DRM and CUDA work in session 0).
import { createRequire } from 'node:module'
import { platform } from 'node:os'
import { isDeepStrictEqual } from 'node:util'
const require = createRequire(import.meta.url)
const addonPath = process.env.LL_BRIDGE_ADDON ?? './livelinguist-core.node'
const { CoreClient } = require(addonPath)

const client = new CoreClient()
const status = JSON.parse(client.runtimeStatus())
const report = JSON.parse(client.probeHardware())
const probe = JSON.parse(client.probeBackends())

const backend = platform() === 'win32' ? 'directx' : platform() === 'darwin' ? 'metal' : 'vulkan'
if (!status.ui || status.ui.backend !== backend) {
  throw new Error(`runtimeStatus.ui.backend expected ${backend}, got ${status.ui?.backend}`)
}
if (status.uiBackend !== undefined) throw new Error('uiBackend must be replaced by the measured ui object')
if (typeof status.ui.apiAvailable !== 'boolean') throw new Error('apiAvailable missing')
if (typeof status.ui.hardware !== 'boolean') throw new Error('hardware missing')
if (status.ui.hardware && status.ui.degradedReason !== null) {
  throw new Error('a hardware adapter must not carry a degraded reason')
}
if (!Array.isArray(report.adapters)) throw new Error('adapters missing')
if (!Array.isArray(report.vulkan.devices) || !Array.isArray(report.cuda.devices)) {
  throw new Error('runtime device arrays missing')
}
if (JSON.stringify(report).includes('"text"')) {
  throw new Error('hardware report must not carry transcript text')
}
for (const adapter of report.adapters) {
  if (adapter.software && adapter.kind !== 'software') {
    throw new Error(`software adapter ${adapter.name} must not be classified as ${adapter.kind}`)
  }
  if (!adapter.software && !['discrete', 'integrated'].includes(adapter.kind)) {
    throw new Error(`hardware adapter ${adapter.name} has kind ${adapter.kind}`)
  }
}
const cpu = probe.devices.find((device) => device.id === 'cpu')
if (!cpu?.available) throw new Error('CPU backend must always be available')
const cuda = probe.devices.find((device) => device.id === 'cuda')
if (Boolean(cuda?.available) !== (report.cuda.devices.length > 0)) {
  throw new Error('CUDA availability must match the measured device list')
}
if (!isDeepStrictEqual(probe.ui, status.ui)) {
  throw new Error('probe.ui and runtimeStatus.ui disagree')
}

console.log(
  JSON.stringify({
    ok: true,
    ui: status.ui.backend,
    hardware: status.ui.hardware,
    adapter: status.ui.adapter,
    adapters: report.adapters.length,
    vulkan: report.vulkan.available,
    cuda: report.cuda.available,
    cudaDevices: report.cuda.devices.map((device) => device.name),
  }),
)
