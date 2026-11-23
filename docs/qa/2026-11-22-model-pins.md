# Model pins (v1.0)

- Date: 2026-11-22
- Status: **pins verified and mirror published**; per-locale measurement pending
- Decision: ADR 0019 (D-056)
- Code: `core/crates/ll-models/src/lib.rs` (`ModelCatalog::plan_default`)

## Verified set

One ASR model and one simplifier. Both are downloaded on first run, never
bundled, and verified before use.

| Role | Artifact | Revision | Bytes | SHA-256 |
|---|---|---|---|---|
| ASR encoder | `ndgold/nemotron-3.5-asr-streaming-0.6b-onnx` / `encoder.int8.onnx` | `f73e3bc8…` | 657,601,403 | `012e9321373af99021415e0b0eb3ec827b4be3153be6f30d9b448fe65e896e68` |
| ASR decoder | same / `decoder.int8.onnx` | `f73e3bc8…` | 14,978,075 | `19f9c98fc6d0a2c33a65a43b36fdb2e914c26c0aa9764be3aebc502a1e982fb0` |
| ASR joiner | same / `joiner.int8.onnx` | `f73e3bc8…` | 9,504,438 | `4101c7c679a0bc30483794b27a059e34e79232aa2068d78d51231a22c8b0d7ce` |
| ASR tokens | same / `tokens.txt` | `f73e3bc8…` | 131,440 | `729cc103155bafa785f9cd45746cd41cabe97eab7182fc04d594129587958f8a` |
| LLM | `ndgold/Qwen3-0.6B-EasyLanguage-GGUF` / `Qwen3-0.6B-EasyLanguage-Q4_K_M.gguf` | `658db5ad6a05e3f549990b10e8d8c59a78866695` | 396,704,608 | `1be8c19cc1153d33535c3ad6a3306be7d84ecc4ae7a171bf6c1b428c6737ffb8` |

Totals: ASR 682,215,356 bytes (~682 MB); LLM 396,704,608 bytes (~397 MB); first
run ~1.08 GB.

Licenses: Nemotron 3.5 ASR is NVIDIA Open Model License (OpenMDW v1.1); the int8
conversion is produced by sherpa-onnx (Apache-2.0 tooling). Qwen3 0.6B
EasyLanguage is Apache-2.0, a LoRA fine-tune of `Qwen/Qwen3-0.6B`. Attribution
lives in `packaging/licenses/THIRD_PARTY.md` and the in-app licenses screen.

## Upstream provenance

- sherpa-onnx release tag `asr-models`, asset
  `sherpa-onnx-nemotron-3.5-asr-streaming-0.6b-560ms-int8-2026-06-11.tar.bz2`
  (asset id 470963822, updated 2026-07-09), 475,271,763 bytes, digest
  `sha256:c6bf5e0df765f9d5b43bc9e0536d4b4b3e7d40bdf5ecf13e45f134c51c05ae3a`.
- NVIDIA model card: `nvidia/nemotron-3.5-asr-streaming-0.6b` (public); the repo
  ships `.nemo` and safetensors, not per-file ONNX, so the sherpa int8 package
  is the runtime artifact.
- `ndgold/nemotron-3.5-asr-streaming-0.6b-onnx` is **public**. It was published
  on 2026-11-22 at commit
  `f73e3bc81614fa69e24b091b892e8672983fbb9c`, with `README.md`,
  `PROVENANCE.md`, and `LICENSE`. Anonymous `resolve` returns 200 for every
  file, and the LFS SHA-256 values match the table.

## Verification performed

1. Queried the Hugging Face API for the GGUF repo and read the LFS SHA-256.
2. Downloaded the sherpa `.tar.bz2` once; its SHA-256 matched the GitHub asset
   digest.
3. Extracted with `tar -xjf`; hashed each file with SHA-256 and recorded sizes.
4. Recorded revision, size, license, and provenance in the catalog and here.
5. Published the four files plus `README.md`, `PROVENANCE.md`, and `LICENSE` to
   the public mirror, then re-read the LFS SHA-256 values from the HF API; all
   match. `tokens.txt` is a regular git blob with no LFS hash, so it was
   downloaded and hashed directly; it matches.
6. Pinned the mirror commit `f73e3bc8…` in the catalog (not `main`).

## Not yet done (release blockers for W-3)

1. Measure WER/CER, RTF, and p95 chunk per locale; promote coverage rows only
   from measurements.
2. Compare 160 ms and 560 ms once on tier C; 560 ms remains the default unless
   it misses the latency gate.
3. Verify the rendered prompt matches the GGUF contract exactly: pre-filled
   empty think block, trailing `/no_think`, no few-shot, no rolling context.
