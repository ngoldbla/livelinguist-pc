# Decisions

One file per decision, numbered, short. Each records the context, the options
considered, the choice, and what measurement would overturn it. Spike results go
in the same file as a dated addendum.

Status labels: `proposed`, `accepted`, `superseded`.

| # | Decision | Status |
|---|---|---|
| 0001 | Primary ASR runtime (sherpa-onnx vs ONNX Runtime vs whisper.cpp) | open — M0 spike 1 |
| 0002 | Simplifier runtime and GGUF conversion path | open — M0 spike 2 |
| 0003 | System-audio capture model (monitor source vs virtual sink) | open — M0 spike 3 |
| 0004 | Overlay strategy on Wayland | open — M0 spike 4 |
| 0005 | Core implementation language | open |
| 0006 | Live budget per hardware tier | open — follows 0001–0002 |

Template:

```md
# 000N — Title

Status: proposed | accepted | superseded by 000M
Date: YYYY-MM-DD

## Context
What forced a decision, and what the reference implementation assumed.

## Options
Each with the evidence for and against.

## Decision
The choice, stated so it can be checked.

## Consequences
What this makes easy, hard, or impossible. What would overturn it.

## Evidence
Commands, hardware, measured numbers, artifact paths.
```
