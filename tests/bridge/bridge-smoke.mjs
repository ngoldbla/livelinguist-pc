#!/usr/bin/env node
// Functional bridge smoke test: commands, snapshot, on-demand simplification,
// monotonic envelope sequence, and content-free exports.
import { createRequire } from 'node:module'
const require = createRequire(import.meta.url)
const addonPath = process.env.LL_BRIDGE_ADDON ?? './livelinguist-core.node'
const { CoreClient } = require(addonPath)

const client = new CoreClient()
const envelopes = []
client.start(JSON.stringify({ language: 'fr-FR', source: 'demo', mode: 'on_demand', detection: true, scenario: 'mock' }), (error, value) => {
  if (error) { console.error(error); process.exit(2) }
  envelopes.push(JSON.parse(value))
})
client.feedDemo()
await new Promise((resolve) => setTimeout(resolve, 50))
const snapshot = JSON.parse(client.snapshot())
if (snapshot.segments.length !== 9) throw new Error(`expected 9 segments, got ${snapshot.segments.length}`)
if (snapshot.lastSeq < 100) throw new Error('too few events')
let last = 0
for (const envelope of envelopes) {
  if (envelope.v !== 1) throw new Error('protocol version')
  if (envelope.seq <= last) throw new Error('sequence not monotonic')
  last = envelope.seq
}
const id = snapshot.segments[0].id
const simplified = JSON.parse(client.simplify(id))
if (!simplified.text) throw new Error('on-demand simplification returned no text')
const summary = JSON.parse(client.stop())
if (summary.segments !== 9) throw new Error('summary segment count')
if (summary.simplified !== 1) throw new Error('summary simplified count')

// Export after stop: the Summary screen exports the session that just ended.
import { mkdtempSync, statSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
const dir = mkdtempSync(join(tmpdir(), 'll-bridge-export-'))
for (const [format, extension] of [['markdown', 'md'], ['plain', 'txt'], ['csv', 'csv']]) {
  const path = join(dir, `summary.${extension}`)
  client.export(format, path)
  if (statSync(path).size === 0) throw new Error(`empty ${format} export after stop`)
}
console.log(JSON.stringify({ ok: true, envelopes: envelopes.length, segments: summary.segments, simplified: summary.simplified }))
