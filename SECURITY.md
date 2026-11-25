# Security

Please report security problems privately through
[GitHub private vulnerability reporting](https://github.com/ngoldbla/livelinguist-pc/security/advisories/new),
not as a public issue.

Only the latest release is supported. Releases are built by GitHub Actions
from the tagged source and published with SHA-256 sums; see
[`packaging/README.md`](packaging/README.md). Release builds are not
code-signed yet (ADR 0020).

Live Linguist never sends audio or transcripts off the device. Its only network
use is the user-started model download from pinned, hash-verified Hugging Face
files.
