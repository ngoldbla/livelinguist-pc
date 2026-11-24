import { prepareEnvironment } from "./bootstrap";

prepareEnvironment();

const { render } = await import("@gpuix/react");
const { App } = await import("./App");
const { readWorkArea, windowTitle } = await import("./bridge/transport");
const { fitWindow } = await import("./windowSize");
const { windowKeys } = await import("./captionKeys");

const screen = (process.env.LL_SCREEN ?? "").toLowerCase();
const [defaultWidth, defaultHeight] =
  screen === "overlay"
    ? [780, 250]
    : screen === "licenses"
      ? [1020, 880]
      : screen === "settings"
        ? [1180, 880]
        : screen === "summary"
          ? [1180, 840]
          : screen === "expanded"
            ? [1180, 900]
            : [1180, 820];

// Fit to the monitor work area so nothing clips at 150% scale. The overlay
// keeps its fixed size; LL_WIDTH/LL_HEIGHT still override everything.
let workArea = null;
try {
  workArea = readWorkArea();
} catch {
  workArea = null;
}
const fitted =
  screen === "overlay"
    ? { width: defaultWidth, height: defaultHeight }
    : fitWindow({ width: defaultWidth, height: defaultHeight }, workArea);
const width = Number(process.env.LL_WIDTH ?? fitted.width);
const height = Number(process.env.LL_HEIGHT ?? fitted.height);

render(<App />, {
  title: windowTitle,
  width,
  height,
  appName: "Live Linguist",
  focus: process.env.GPUIX_BACKGROUND !== "1",
  titlebarTransparent: true,
  windowBackground: "opaque",
  // Window-level shortcuts (Ctrl+Shift+L: Read latest) go through a small bus
  // the screens subscribe to.
  onKeyDown: (event) => {
    if (windowKeys.dispatch(event.key ?? "", event.modifiers)) event.preventDefault();
  },
});
