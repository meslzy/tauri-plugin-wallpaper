import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow, type WindowLabel } from "@tauri-apps/api/window";

enum Action {
  Attach = "attach",
  Detach = "detach",
  Reset = "reset",
}

const action = (action: Action) => `plugin:wallpaper|${action}`;

/**
 * Attaches window to the desktop.
 * @param windowLabel The window label to attach the desktop to. If not provided, the current window will be used.
 */
export const attach = (windowLabel?: WindowLabel) => {
  if (!windowLabel) {
    windowLabel = getCurrentWindow().label;
  }

  return invoke(action(Action.Attach), {
    payload: {
      windowLabel,
    },
  });
};

/**
 * Detaches window from the desktop.
 * @param windowLabel The window label to detach the desktop from. If not provided, the current window will be used.
 */
export const detach = (windowLabel?: WindowLabel) => {
  if (!windowLabel) {
    windowLabel = getCurrentWindow().label;
  }

  return invoke(action(Action.Detach), {
    payload: {
      windowLabel,
    },
  });
};

/**
 * Resets the wallpaper to the default.
 */
export const reset = () => {
  return invoke(action(Action.Reset));
};

export default {
  attach,
  detach,
  reset,
};
