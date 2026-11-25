import { Surface } from "../aurora/components";
import { colors, textStyle } from "../aurora/theme";

// Shown by a packaged build when livelinguist-core.node cannot load (ADR 0021):
// never a silent fallback to the demo transport. English only: this screen
// runs before the locale and the bridge exist. The detail is the loader's
// error message (a file path and an OS reason), never user content.
export function StartupErrorScreen({ detail }: { detail: string }) {
  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        gap: 16,
        padding: 28,
        height: "100%",
        backgroundColor: colors.bg,
      }}
    >
      <text style={textStyle(24, colors.text, 700)}>Live Linguist could not start</text>
      <text style={textStyle(15, colors.text)}>
        A program file is missing or was blocked by security software. Uninstall Live Linguist,
        download the installer again from the project's Releases page, and install it.
      </text>
      <text style={textStyle(15, colors.muted)}>
        If it still fails, open an issue on the project page and include the details below.
      </text>
      <Surface>
        <text style={textStyle(13, colors.muted)}>{detail}</text>
      </Surface>
    </div>
  );
}
