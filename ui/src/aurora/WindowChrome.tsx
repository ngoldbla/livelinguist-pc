import { BrandMark, Icon } from "./components";
import type { IconName } from "./iconData";
import { colors, textStyle, type } from "./theme";

/**
 * Custom title bar for the frameless GPUIX window: a drag region that starts
 * the native move loop, double-click to maximize, and the three window
 * buttons. Rendered everywhere except the caption overlay, which keeps its own
 * Move control.
 */
export function WindowChrome({
  onDrag,
  onMinimize,
  onToggleMax,
  onClose,
}: {
  onDrag: () => void;
  onMinimize: () => void;
  onToggleMax: () => void;
  onClose: () => void;
}) {
  return (
    <div
      style={{
        height: 34,
        flexShrink: 0,
        display: "flex",
        flexDirection: "row",
        alignItems: "stretch",
        backgroundColor: colors.bgDeep,
        borderBottomWidth: 1,
        borderColor: colors.border,
        userSelect: "none",
      }}
    >
      <div
        onMouseDown={(event) => {
          if (event.clickCount === 2) onToggleMax();
          else onDrag();
        }}
        style={{
          display: "flex",
          flexDirection: "row",
          alignItems: "center",
          gap: 8,
          flexGrow: 1,
          minWidth: 0,
          paddingLeft: 10,
          cursor: "default",
        }}
      >
        <BrandMark size={18} />
        <text style={{ ...textStyle(type.meta, colors.faint, 600), whiteSpace: "nowrap" }}>
          Live Linguist
        </text>
      </div>
      <ChromeButton label="Minimize" icon="minus" onClick={onMinimize} />
      <ChromeButton label="Maximize or restore" icon="square" onClick={onToggleMax} />
      <ChromeButton label="Close" icon="x" onClick={onClose} close />
    </div>
  );
}

function ChromeButton({
  label,
  icon,
  onClick,
  close = false,
}: {
  label: string;
  icon: IconName;
  onClick: () => void;
  close?: boolean;
}) {
  return (
    <div
      role="button"
      tabIndex={0}
      aria-label={label}
      onClick={onClick}
      onKeyDown={(event) => {
        if (event.key === "enter" || event.key === "space" || event.key === " ") {
          event.preventDefault();
          onClick();
        }
      }}
      style={{
        width: 44,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        cursor: "default",
        hover: { backgroundColor: close ? "#C42B1C" : "#FFFFFF12" },
        active: { backgroundColor: close ? "#A62417" : "#FFFFFF1C" },
        focusVisible: { outlineWidth: 2, outlineColor: colors.accent, outlineOffset: -2 },
      }}
    >
      <Icon name={icon} size={14} color={close ? colors.text : colors.muted} />
    </div>
  );
}
