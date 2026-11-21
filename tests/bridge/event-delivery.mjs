#!/usr/bin/env node
// S-2 event-delivery harness: 100 events/s through the napi TSFN.
// Usage: node event-delivery.mjs [seconds] [out.json]
// Run under Node and Bun; p99 drift must stay under 50 ms with no drops.

import { createRequire } from 'node:module'
const require = createRequire(import.meta.url)
const addonPath = process.env.LL_BRIDGE_ADDON ?? './livelinguist-core.node'
const { CoreClient } = require(addonPath)

const seconds = Number(process.argv[2] ?? 10)
const outPath = process.argv[3] ?? null
const rate = 100
const expected = rate * seconds
const client = new CoreClient()
const samples = []
let count = 0
let firstArrival = 0
let firstTs = 0
let resolveDone
const done = new Promise((resolve) => { resolveDone = resolve })

client.start(JSON.stringify({ language: 'fr-FR', source: 'demo', mode: 'on_demand', scenario: 'mock' }), (error, value) => {
  if (error) { console.error('bridge error', error); process.exit(2) }
  const envelope = JSON.parse(value)
  if (envelope.event.type !== 'log' || envelope.event.code !== 'delivery_test') return
  const now = performance.now()
  count += 1
  if (firstArrival === 0) {
    firstArrival = now
    firstTs = envelope.ts
    return
  }
  const drift = (now - firstArrival) - (envelope.ts - firstTs)
  samples.push(drift)
  if (count >= expected) resolveDone()
})
client.deliveryTest(rate, seconds)
const watchdog = setTimeout(() => resolveDone(), seconds * 1000 + 5000)
await done
clearTimeout(watchdog)
client.stopDeliveryTest()
client.stop()

function percentile(values, p) {
  if (values.length === 0) return null
  const sorted = [...values].sort((a, b) => a - b)
  const rank = Math.max(1, Math.ceil((p / 100) * sorted.length))
  return sorted[Math.min(rank - 1, sorted.length - 1)]
}
const result = {
  runtime: typeof Bun !== 'undefined' ? `bun ${Bun.version}` : `node ${process.version}`,
  rate,
  seconds,
  expected,
  received: count,
  dropped: expected - count,
  p50Ms: percentile(samples, 50),
  p99Ms: percentile(samples, 99),
  maxMs: samples.length ? Math.max(...samples) : null,
}
const text = JSON.stringify(result, null, 2)
console.log(text)
if (outPath) {
  const fs = await import('node:fs')
  fs.writeFileSync(outPath, text + '\n')
}
const failed = result.dropped > 0 || result.p99Ms === null || result.p99Ms > 50
process.exit(failed ? 1 : 0)
