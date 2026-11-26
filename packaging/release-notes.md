## What's new in this build

- **Microphone input.** Choose **Microphone** under Sound and Live Linguist
  captions speech from your microphone or headset as well as from your
  computer's audio.
- **Choose the device.** A **Device** menu under Sound lists your Windows output
  or input devices, names the current default, and remembers your choice for
  next time.
- **No surprise device swaps.** A device you choose explicitly stays chosen. If
  it is unplugged, the app says so and uses the Windows default for that session
  instead of failing silently or capturing something else.
- Audio devices that do not report float samples now work too; the app converts
  them for the caption pipeline.
- Everything from the previous candidate remains: the simpler Mac-shaped
  console, 24px simplified captions, the window controls, all 29 language packs,
  and click-only simplification.

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

**Privacy:** audio and captions never leave your PC. Microphone audio is
processed on this PC exactly like system audio and is never recorded or sent.
The only network use is the one-time model download from Hugging Face.

**Uninstall:** Settings → Apps → Installed apps → Live Linguist → Uninstall.
Your downloaded models are kept unless you tick the box to delete them.

**No installer?** `LiveLinguist-{{VERSION}}-portable.zip` runs without
installing: unzip it into a folder and open `livelinguist.exe`.

**Something didn't work?** [Open an issue](https://github.com/{{REPO}}/issues/new/choose).
Never paste captions or transcripts into an issue.

### Code signing policy

Free code signing is provided by [SignPath.io](https://signpath.io), certificate
by [SignPath Foundation](https://signpath.org). Builds published so far are
unsigned; once signing is live, Windows shows **SignPath Foundation** as the
publisher. Only binaries built from this repository by GitHub Actions are
signed, and a maintainer approves each signing request. Signed files, team roles
and verification steps: [`CODE_SIGNING.md`](https://github.com/{{REPO}}/blob/main/CODE_SIGNING.md).

### Check the download (optional)

These files were built by GitHub Actions from the tagged source. To check that
your download is unchanged, open PowerShell in your Downloads folder and run
`Get-FileHash .\LiveLinguist-{{VERSION}}-setup.exe`. The hash must match the
line below (letter case does not matter). With the GitHub CLI you can also check
the build provenance: `gh attestation verify LiveLinguist-{{VERSION}}-setup.exe --repo {{REPO}}`.

