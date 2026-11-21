# Skills lock (T-014 / D-047)

Policy: skills are vendored into `.opencode/skills/` at a reviewed state; no
floating `npx skills add` and no runtime download in CI. Maximum three external
skills, two custom skills.

## Custom skills (vendored, reviewed)

| Skill | Path | SHA-256 | Reviewer | Date |
|---|---|---|---|---|
| gpuix | `.opencode/skills/gpuix/SKILL.md` | `fcf986bcb2cc2681d4797779bd270189db5c30e82466f276158ba77286384e98` | repository maintainer (agent) | 2026-11-21 |
| live-linguist-core | `.opencode/skills/live-linguist-core/SKILL.md` | `f0df4dabcb57b56e096ce4346840297d001d2b32eecabb516dc05a5ae794d196` | repository maintainer (agent) | 2026-11-21 |

## External skills (max 3)

**None installed for v1.** The two custom skills cover the project-specific
rules; external React/Rust skills would add supply-chain surface without
project-specific value. If one is added later, record repo + commit SHA +
reviewer + date here before it is used in CI.

Candidates not installed (from plan §21.2):

- `vercel-labs/agent-skills/vercel-react-best-practices`
- `apollographql/skills/rust-best-practices`
- `po4yka/rust-skills/rust-port`

## Forbidden skill sources

`longbridge/gpui-kit`, `terraphim/gpui-components`, random `gpui` skills,
Tailwind/shadcn/Radix/Framer Motion/web design-system skills, DOM/Playwright
testing skills, cloud STT, Tauri packaging, Flutter/mobile, `gpui-fast`.
