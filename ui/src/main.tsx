import { prepareEnvironment } from "./bootstrap";

prepareEnvironment();

const { render } = await import("@gpuix/react");
const { App } = await import("./App");
const { windowTitle } = await import("./bridge/transport");

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
          : [1180, 820];

const width = Number(process.env.LL_WIDTH ?? defaultWidth);
const height = Number(process.env.LL_HEIGHT ?? defaultHeight);

render(<App />, {
  title: windowTitle,
  width,
  height,
  appName: "Live Linguist",
  focus: process.env.GPUIX_BACKGROUND !== "1",
  titlebarTransparent: true,
  windowBackground: "opaque",
});
