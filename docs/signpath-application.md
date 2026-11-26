# SignPath Foundation: application and setup

Decision: [ADR 0020](decisions/0020.md). Policy users see:
[`CODE_SIGNING.md`](../CODE_SIGNING.md). Integration: [`.signpath/`](../.signpath/README.md).

**State:** submitted 2026-11-26, awaiting review. Nothing is signed yet; the
repository side below is already on `main` (PR #47) and the `v1.0.0-rc.5`
release notes carry the "Code signing policy" section that the download URL
points to.

### What was filed

| Field | Value |
| --- | --- |
| Project | Live Linguist PC |
| Repository / homepage | `github.com/ngoldbla/livelinguist-pc` |
| Download page | `github.com/ngoldbla/livelinguist-pc/releases` |
| Privacy policy | `README.md#privacy` (the app collects nothing) |
| Category | Accessibility |
| Maintainer type | For-profit company or corporate-backed project (Aquilops LLC) |
| Build system | GitHub Actions |
| Discovery | Organic search (Google) |
| Account | Dylan Goldblatt, `dylan@aquilops.org`, Aquilops LLC |

The **Reputation** field says plainly that the project is new: public since
21 November 2026, five tagged pre-releases in five days, no adoption metrics
yet, and a port of the maintainer's macOS/iOS implementation in development
since June 2026. Nothing was dressed up; the public download counts are
single-digit and reviewers can see them.

Two required consents were ticked on the owner's instruction: the SignPath
Foundation Code of Conduct (certificates are issued in the Foundation's name and
can be revoked) and storage/processing of the personal data in the form. The
optional marketing consent was left unticked.

## 1. Apply (owner, 10 minutes)


Open <https://signpath.org/apply> and answer with the text below. The form is
short; anything unclear goes in "anything else".

| Field | Answer |
| --- | --- |
| Project / repository | `https://github.com/ngoldbla/livelinguist-pc` |
| License | MIT (`LICENSE` at the repository root, OSI-approved) |
| Download / release page | `https://github.com/ngoldbla/livelinguist-pc/releases` |
| Code signing policy | `https://github.com/ngoldbla/livelinguist-pc/blob/main/CODE_SIGNING.md` |
| Build system | GitHub Actions, `.github/workflows/release.yml`, GitHub-hosted `windows-2025` runners |
| Artifacts to sign | Windows x64: `livelinguist.exe`, `livelinguist-core.node`, `LiveLinguist-<version>-setup.exe` (NSIS) |
| Maintainer | your name and the e-mail you want reviewers to use |

Project description, ready to paste:

> Live Linguist PC is a free Windows 11 desktop app that captions system audio
> or a microphone live, on the device, and rewrites any sentence in the same
> language as easy language. Speech recognition (sherpa-onnx) and the
> simplifier (llama.cpp with a Qwen3 0.6B fine-tune) run locally on the CPU;
> audio, captions and transcripts never leave the user's PC and no analytics
> are collected. It is meant for people who need easier-to-read text: early
> readers, people with dyslexia or aphasia, deaf and hard-of-hearing users, and
> language learners following a call or a class. It ships as a per-user NSIS
> installer (`LiveLinguist-<version>-setup.exe`, no admin rights) and a portable
> ZIP, both built by GitHub Actions from the tagged source of this repository.
> The binaries built from our source are `livelinguist.exe` and
> `livelinguist-core.node`, plus the setup executable; the GPUIX runtime is a
> third-party Apache-2.0 binary redistributed unchanged and is not re-signed.
> The project is MIT-licensed, has no commercial edition, no paid tier and no
> proprietary components.

Reviewers may ask follow-up questions by e-mail. Answer them; the reputation
check for a young project is the usual reason an application takes a while.

## 2. Why this project qualifies

| Condition (signpath.org/terms) | Evidence |
| --- | --- |
| OSI-approved license, no commercial dual licensing | MIT, `LICENSE`; no second license, no paid edition |
| No proprietary components | `packaging/licenses/THIRD_PARTY.md`: GPUIX, GPUI, React, Bun, llama.cpp, sherpa-onnx are all OSS. The NVIDIA ASR model license applies to downloaded model files, which are not part of any signed artifact |
| Actively maintained | Commits and releases through `v1.0.0-rc.5` |
| Already released in the form to be signed | `setup.exe` and portable ZIP published on GitHub Releases |
| Documented on the download page | `README.md` "Download for Windows", release notes template |
| No malware, PUP, or exploitation tooling | An accessibility/captioning app; it captures audio the user selects and nothing else |
| Sign only own projects and own binaries | One owner holds the repo and the SignPath project; the only redistributed binary is GPUIX, left unsigned |
| Privacy: nothing sent unless the user asks | `CODE_SIGNING.md` "Privacy": model download on click, update check off by default, no telemetry |
| Uninstaller, no silent system changes | NSIS per-user install, uninstaller registered, portable ZIP writes no registry keys |
| MFA for every team member | Owner enables MFA on GitHub (and later on SignPath) |
| Code signing policy published | `CODE_SIGNING.md`, linked from `README.md` and from every release page |
| Authors / reviewers / approvers named | `CODE_SIGNING.md` role table |
| Every signing request approved by hand | `release-signing` policy requires maintainer approval; the CI token cannot approve |

## 3. After approval (owner, about 15 minutes)

1. Accept the invitation, sign in at <https://app.signpath.io>, enable MFA.
2. **Organization → Trusted build systems:** add the predefined **GitHub.com**
   connector; restrict it to `ngoldbla/livelinguist-pc` and to
   `.github/workflows/release.yml`. Install the SignPath GitHub App on the
   repository when prompted.
3. **Project** slug `livelinguist-pc`, with two signing policies:
   - `test-signing` — SignPath's test certificate, for the setup run.
   - `release-signing` — production certificate, **manual approval required**,
     origin restricted as in step 2.
4. **Artifact configurations:** create both and paste the XML verbatim from
   `.signpath/artifact-configurations/`:
   - `windows-payload` ← `windows-payload.xml`
   - `windows-installer` ← `windows-installer.xml`
5. **CI user:** create a user with submitter rights on the project, *no*
   approval rights, and issue an API token.
6. In GitHub under **Settings → Secrets and variables → Actions**:
   - Secret `SIGNPATH_API_TOKEN` — the token from step 5.
   - Variable `SIGNPATH_ORGANIZATION_ID` — the organization ID.
   - Variable `SIGNPATH_PROJECT_SLUG` — `livelinguist-pc`.
   - Variable `SIGNPATH_SIGNING_POLICY_SLUG` — `test-signing`.

## 4. Test run with the test certificate

1. **Actions → Windows release → Run workflow**, on `main`, "Publish a
   release" **unticked**. Nothing is published.
2. The run stops twice waiting for approval. Approve both requests in SignPath
   after checking the commit and the artifacts.
3. The run prints the signer of each file. The test certificate is trusted only
   on the runner; the `publish` job refuses to publish while the policy is
   `test-signing`.
4. Send SignPath the successful run URL and ask for the production certificate.
   They review the setup and issue it against `release-signing`.

## 5. First signed release

1. Set `SIGNPATH_SIGNING_POLICY_SLUG` to `release-signing`.
2. Bump the version, tag `v1.0.0-rc.6`, approve the two signing requests.
3. `publish` fails loudly if the build was not signed, or if the policy is
   still `test-signing`.
4. Check the published installer on a normal PC:

```powershell
Get-AuthenticodeSignature .\LiveLinguist-1.0.0-rc.6-setup.exe |
  Format-List Status, SignerCertificate
```

`Status` must be `Valid` and the subject must contain `CN=SignPath Foundation`.
The SmartScreen prompt may still appear on the first downloads of a new
certificate: reputation builds with download volume, it is not granted by
signing.

## 6. Things to watch on the first request

- **Bun-compiled executable.** `livelinguist.exe` is produced by
  `bun build --compile`, which appends the bundled source to the PE. Authenticode
  covers appended bytes, but confirm after the first signing request that the
  file still starts (the install/self-test step does this) and that its size
  changed only by the signature block.
- **Installer metadata.** `product-name="Live Linguist"` is asserted in both
  artifact configurations. If SignPath reports a mismatch, it is the version
  resource in `packaging/nsis/livelinguist.nsi`; adjust the attribute rather
  than the product name. Add a `product-version` constraint once the first
  request shows the value SignPath reads.
- **Uninstaller.** `uninstall.exe` is generated by NSIS at install time and
  stays unsigned. This is documented in `CODE_SIGNING.md`.
- **Portable ZIP.** ZIP archives carry no Authenticode signature; the files
  inside it are signed.
- **Approval latency.** Every request waits for a human. A release tag therefore
  stays "in progress" until the two requests are approved.
