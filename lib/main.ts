import { invoke } from "@tauri-apps/api/core";
import { WindowLabel, getCurrentWindow } from "@tauri-apps/api/window";

enum Action {
  Attach = "attach",
  Detach = "detach",
  Reset = "reset",
}

const currentWindow = getCurrentWindow();

const action = (action: Action) => `plugin:wallpaper|${action}`;

/**
 * Attaches window to the desktop.
 * @param windowLabel The window label to attach the desktop to. If not provided, the current window will be used.
 */
const attach = (windowLabel?: WindowLabel) => {
  if (!windowLabel) {
    windowLabel = currentWindow.label;
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
const detach = (windowLabel?: WindowLabel) => {
  if (!windowLabel) {
    windowLabel = currentWindow.label;
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
const reset = () => {
  return invoke(action(Action.Reset));
};

const wallpaper = {
  /**
   * Attaches window to the desktop.
   * @param windowLabel The window label to attach the desktop to. If not provided, the current window will be used.
   **/
  attach,
  /**
   * Detaches window from the desktop.
   * @param windowLabel The window label to detach the desktop from. If not provided, the current window will be used.
   **/
  detach,
  /**
   * Resets the wallpaper to the default.
   **/
  reset,
};

export {
  attach,
  detach,
  reset,
};

export default wallpaper;
