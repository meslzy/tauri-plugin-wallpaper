import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow, type WindowLabel } from "@tauri-apps/api/window";

enum Action {
  Attach = "attach",
  Detach = "detach",
  Reset = "reset",
  Pin = "pin",
  Unpin = "unpin",
  Capabilities = "capabilities",
  IsAttached = "is_attached",
  IsPinned = "is_pinned",
  SetInteractive = "set_interactive",
  StartOcclusionMonitor = "start_occlusion_monitor",
  StopOcclusionMonitor = "stop_occlusion_monitor",
  SetWallpaperImage = "set_wallpaper_image",
  GetWallpaperImage = "get_wallpaper_image",
}

const action = (action: Action) => `plugin:wallpaper|${action}`;

const currentLabel = () => getCurrentWindow().label;

/**
 * What the current platform supports.
 * Use this to feature-detect instead of relying on rejected promises.
 */
export interface Capabilities {
  /** The current platform (e.g. "windows", "macos", "linux"). */
  platform: string;
  /** Whether attaching a window behind desktop icons is supported. */
  attach: boolean;
  /** Whether detaching is supported. */
  detach: boolean;
  /** Whether resetting the wallpaper is supported (Windows only). */
  reset: boolean;
  /** Whether pinning (always-on-top surviving Show Desktop) is supported. */
  pin: boolean;
  /** Whether unpinning is supported. */
  unpin: boolean;
  /** Whether raw input forwarding to wallpaper windows is supported (Windows only). */
  inputForwarding: boolean;
  /** Whether setInteractive is supported. */
  interactive: boolean;
  /** Whether occlusion monitoring is supported. */
  occlusion: boolean;
  /** Whether setting/getting the OS wallpaper image is supported. */
  wallpaperImage: boolean;
}

/** Options for {@link attach}. */
export interface AttachOptions {
  /** The window to attach. Defaults to the current window. */
  windowLabel?: WindowLabel;
  /** Forward global mouse input to the wallpaper window (Windows only). */
  forwardMouseInput?: boolean;
  /** Forward global keyboard input to the wallpaper window (Windows only). */
  forwardKeyboardInput?: boolean;
  /** Name of the monitor to cover (from `availableMonitors()`); defaults to the current bounds. */
  monitor?: string;
}

/** Payload of attached/detached/pinned/unpinned/reattached events. */
export interface WindowEventPayload {
  windowLabel: string;
}

/** Payload of the occlusion event. */
export interface OcclusionEventPayload {
  windowLabel: string;
  occluded: boolean;
}

/** Event names emitted by the plugin. */
export const WallpaperEvent = {
  Attached: "wallpaper://attached",
  Detached: "wallpaper://detached",
  Pinned: "wallpaper://pinned",
  Unpinned: "wallpaper://unpinned",
  /** A window was automatically re-attached after explorer.exe restarted (Windows). */
  Reattached: "wallpaper://reattached",
  /** Occlusion state of an attached window changed (requires startOcclusionMonitor). */
  Occlusion: "wallpaper://occlusion",
} as const;

/**
 * Returns what the current platform supports.
 * On Linux this is a runtime check: X11 sessions are supported, Wayland is not.
 */
export const capabilities = () => {
  return invoke<Capabilities>(action(Action.Capabilities));
};

/**
 * Attaches a window to the desktop, behind the icons.
 * @param windowLabelOrOptions A window label, an options object, or nothing for the current window.
 */
export const attach = (windowLabelOrOptions?: WindowLabel | AttachOptions) => {
  const options =
    typeof windowLabelOrOptions === "string"
      ? {
          windowLabel: windowLabelOrOptions,
        }
      : (windowLabelOrOptions ?? {});

  return invoke(action(Action.Attach), {
    payload: {
      windowLabel: options.windowLabel ?? currentLabel(),
      forwardMouseInput: options.forwardMouseInput ?? false,
      forwardKeyboardInput: options.forwardKeyboardInput ?? false,
      monitor: options.monitor ?? null,
    },
  });
};

/**
 * Detaches window from the desktop.
 * @param windowLabel The window label to detach. If not provided, the current window will be used.
 */
export const detach = (windowLabel?: WindowLabel) => {
  return invoke(action(Action.Detach), {
    payload: {
      windowLabel: windowLabel ?? currentLabel(),
    },
  });
};

/**
 * Resets the wallpaper to the default (Windows only).
 */
export const reset = () => {
  return invoke(action(Action.Reset));
};

/**
 * Pins window to stay always on top and survive Show Desktop (Win+D / Mission Control).
 * @param windowLabel The window label to pin. If not provided, the current window will be used.
 */
export const pin = (windowLabel?: WindowLabel) => {
  return invoke(action(Action.Pin), {
    payload: {
      windowLabel: windowLabel ?? currentLabel(),
    },
  });
};

/**
 * Unpins window, removing always-on-top and Show Desktop protection.
 * @param windowLabel The window label to unpin. If not provided, the current window will be used.
 */
export const unpin = (windowLabel?: WindowLabel) => {
  return invoke(action(Action.Unpin), {
    payload: {
      windowLabel: windowLabel ?? currentLabel(),
    },
  });
};

/**
 * Whether the window is currently attached as wallpaper.
 */
export const isAttached = (windowLabel?: WindowLabel) => {
  return invoke<boolean>(action(Action.IsAttached), {
    payload: {
      windowLabel: windowLabel ?? currentLabel(),
    },
  });
};

/**
 * Whether the window is currently pinned.
 */
export const isPinned = (windowLabel?: WindowLabel) => {
  return invoke<boolean>(action(Action.IsPinned), {
    payload: {
      windowLabel: windowLabel ?? currentLabel(),
    },
  });
};

/**
 * Makes an attached wallpaper window interactive (or passive again).
 * Windows: toggles full input forwarding. macOS: temporarily raises the
 * window to the normal level so it can be used, then sends it back.
 */
export const setInteractive = (interactive: boolean, windowLabel?: WindowLabel) => {
  return invoke(action(Action.SetInteractive), {
    payload: {
      windowLabel: windowLabel ?? currentLabel(),
      interactive,
    },
  });
};

/**
 * Starts emitting {@link WallpaperEvent.Occlusion} events for attached
 * windows (1s poll), so rendering can be paused while covered by a
 * fullscreen app.
 */
export const startOcclusionMonitor = () => {
  return invoke(action(Action.StartOcclusionMonitor));
};

/**
 * Stops occlusion monitoring.
 */
export const stopOcclusionMonitor = () => {
  return invoke(action(Action.StopOcclusionMonitor));
};

/**
 * Sets the OS wallpaper image (the real wallpaper, not a window).
 * @param path Absolute path to the image file.
 */
export const setWallpaperImage = (path: string) => {
  return invoke(action(Action.SetWallpaperImage), {
    payload: {
      path,
    },
  });
};

/**
 * Returns the path/URI of the current OS wallpaper image.
 */
export const getWallpaperImage = () => {
  return invoke<string>(action(Action.GetWallpaperImage));
};

const onWindowEvent = (event: string, handler: (payload: WindowEventPayload) => void) => {
  return listen<WindowEventPayload>(event, (e) => handler(e.payload));
};

/** Listens for windows being attached as wallpaper. */
export const onAttached = (handler: (payload: WindowEventPayload) => void): Promise<UnlistenFn> =>
  onWindowEvent(WallpaperEvent.Attached, handler);

/** Listens for windows being detached. */
export const onDetached = (handler: (payload: WindowEventPayload) => void): Promise<UnlistenFn> =>
  onWindowEvent(WallpaperEvent.Detached, handler);

/** Listens for windows being pinned. */
export const onPinned = (handler: (payload: WindowEventPayload) => void): Promise<UnlistenFn> =>
  onWindowEvent(WallpaperEvent.Pinned, handler);

/** Listens for windows being unpinned. */
export const onUnpinned = (handler: (payload: WindowEventPayload) => void): Promise<UnlistenFn> =>
  onWindowEvent(WallpaperEvent.Unpinned, handler);

/** Listens for automatic re-attachment after an explorer.exe restart (Windows). */
export const onReattached = (handler: (payload: WindowEventPayload) => void): Promise<UnlistenFn> =>
  onWindowEvent(WallpaperEvent.Reattached, handler);

/** Listens for occlusion changes (requires {@link startOcclusionMonitor}). */
export const onOcclusionChanged = (handler: (payload: OcclusionEventPayload) => void): Promise<UnlistenFn> =>
  listen<OcclusionEventPayload>(WallpaperEvent.Occlusion, (e) => handler(e.payload));

export default {
  attach,
  detach,
  reset,
  pin,
  unpin,
  capabilities,
  isAttached,
  isPinned,
  setInteractive,
  startOcclusionMonitor,
  stopOcclusionMonitor,
  setWallpaperImage,
  getWallpaperImage,
  onAttached,
  onDetached,
  onPinned,
  onUnpinned,
  onReattached,
  onOcclusionChanged,
};
