## What's new in this build

- Captions start faster: the live ASR input chunk is now 560 ms instead of
  1100 ms, which halves the recognizer's chunk latency without changing accuracy
  or CPU use. The overlay partial is still shown above the latest final.
- Clicking a sentence no longer pauses live captions. The simplification runs on
  a worker while audio and captions keep flowing, and the simplified line
  appears when it is ready.
- Caption latency is now measured end to end against a pinned, openly licensed
  video, so future changes can be compared on the same numbers.

## Install on Windows 11

1. Under **Assets** below, click **`LiveLinguist-{{VERSION}}-setup.exe`** to download it.
2. Open the downloaded file. You do not need administrator rights.
3. Windows may show **"Windows protected your PC"**. This happens because the
   app is not code-signed yet (free signing for open-source projects is on the
   way). Click **More info**, then **Run anyway**.
4. Click **Install**. Live Linguist starts when setup finishes.
5. The first time, click **Download** to get the speech and simplification
   models (about 1.1 GB, once). After that it works offline.

**Needs:** Windows 11 on a 64-bit Intel or AMD PC, 8 GB of memory, about
2.5 GB of free disk space, and an internet connection for the first download.

**Privacy:** audio and captions never leave your PC. The only network use is
the one-time model download from Hugging Face.

**Uninstall:** Settings → Apps → Installed apps → Live Linguist → Uninstall.
Your downloaded models are kept unless you tick the box to delete them.

**No installer?** `LiveLinguist-{{VERSION}}-portable.zip` runs without
installing: unzip it into a folder and open `livelinguist.exe`.

**Something didn't work?** [Open an issue](https://github.com/{{REPO}}/issues/new/choose).
Never paste captions or transcripts into an issue.

### Check the download (optional)

These files were built by GitHub Actions from the tagged source. To check that
your download is unchanged, open PowerShell in your Downloads folder and run
`Get-FileHash .\LiveLinguist-{{VERSION}}-setup.exe`. The hash must match the
line below (letter case does not matter). With the GitHub CLI you can also check
the build provenance: `gh attestation verify LiveLinguist-{{VERSION}}-setup.exe --repo {{REPO}}`.

