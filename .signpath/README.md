# SignPath integration

SignPath signs the Windows release. This folder holds the artifact
configurations that are pasted into the SignPath project; they are versioned
here so a change to the packaging layout is reviewed like code.

## Slugs

These names are referenced by `.github/workflows/release.yml`. If the SignPath
project uses different ones, change the workflow variables, not these files.

| Slug | Where | Purpose |
| --- | --- | --- |
| project | SignPath project | `livelinguist-pc` |
| `release-signing` | Signing policy | Production certificate, manual approval per request |
| `test-signing` | Signing policy | SignPath-provided test certificate, used once during setup |
| `windows-payload` | Artifact configuration | `windows-payload.xml` |
| `windows-installer` | Artifact configuration | `windows-installer.xml` |

## Order of operations

Signing is two requests, because Authenticode changes a file's bytes:

1. Build the payload, submit `livelinguist.exe` and `livelinguist-core.node`,
   get signed copies back.
2. Package the NSIS installer and the portable ZIP from those signed files.
3. Submit `LiveLinguist-<version>-setup.exe`, get the signed installer back.
4. Recompute `SHA256SUMS.txt`, attest the provenance, publish.

Nothing is packaged after step 3. The NSIS uninstaller is generated at install
time and stays unsigned; see `CODE_SIGNING.md`.

## Variables the workflow reads

Set under **Settings → Secrets and variables → Actions**. While
`SIGNPATH_PROJECT_SLUG` is empty every signing step is skipped and releases are
published unsigned, exactly as before.

| Name | Kind | Value |
| --- | --- | --- |
| `SIGNPATH_API_TOKEN` | secret | API token of a CI user with submitter rights on `release-signing` |
| `SIGNPATH_ORGANIZATION_ID` | variable | Organization ID from SignPath |
| `SIGNPATH_PROJECT_SLUG` | variable | `livelinguist-pc` |
| `SIGNPATH_SIGNING_POLICY_SLUG` | variable | `release-signing` (or `test-signing` during setup) |

The token user may submit but must not approve. Approval is a maintainer action
in the SignPath UI, as `CODE_SIGNING.md` states.
