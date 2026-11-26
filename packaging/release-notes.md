## What's new in this build

- A simpler interface, shaped like the Mac and iPhone apps: choose your
  language, choose what to listen to, and press Start. The transcript is the
  screen.
- Easier to read: simplified captions start at 24px in a rounded face, nothing
  in the interface is smaller than 12px, and the labels use plain words instead
  of engineering terms.
- The window now behaves like a window: drag the top bar to move it,
  double-click to maximize, and use the minimize, maximize and close buttons.
- Language and sound open from their tiles, the app language can follow your
  learning language or your device, and preferences are kept for next time.
- The interface now carries the Lucide icon set, the twin-L brand mark, and the
  amber "English slipped in" border during a language exchange.
- Captions still start quickly: the 560 ms live chunk and click-to-simplify on a
  worker are unchanged from the last candidate.

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

