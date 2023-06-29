import { invoke, window } from "@tauri-apps/api";

enum Action {
  Attach = "attach",
  Detach = "detach"
}

const action = (action: Action) => `plugin:wallpaper|${action}`;

/**
 * Attaches window to the desktop.
 * @param windowLabel The window label to attach the desktop to. If not provided, the current window will be used.
 **/
const attach = (windowLabel?: window.WindowLabel) => {
  if ( !windowLabel ) {
    windowLabel = window.getCurrent().label;
  }

  return invoke(action(Action.Attach), {
    windowLabel,
  });
};

/**
 * Detaches window from the desktop.
 * @param windowLabel The window label to detach the desktop from. If not provided, the current window will be used.
 **/
const detach = (windowLabel?: window.WindowLabel) => {
  if ( !windowLabel ) {
    windowLabel = window.getCurrent().label;
  }

  return invoke(action(Action.Detach), {
    windowLabel,
  });
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
};

export {
  attach,
  detach,
};

export default wallpaper;