import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}));

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: vi.fn(() => ({
    label: "current-window",
  })),
}));

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  attach,
  capabilities,
  detach,
  getWallpaperImage,
  isAttached,
  isPinned,
  onAttached,
  onOcclusionChanged,
  onReattached,
  pin,
  reset,
  setInteractive,
  setWallpaperImage,
  startOcclusionMonitor,
  stopOcclusionMonitor,
  unpin,
  WallpaperEvent,
} from "./main";

const invokeMock = vi.mocked(invoke);
const listenMock = vi.mocked(listen);

beforeEach(() => {
  invokeMock.mockClear();
  listenMock.mockClear();
});

describe("attach", () => {
  it("accepts a window label", async () => {
    await attach("wallpaper");
    expect(invokeMock).toHaveBeenCalledWith("plugin:wallpaper|attach", {
      payload: {
        windowLabel: "wallpaper",
        forwardMouseInput: false,
        forwardKeyboardInput: false,
        monitor: null,
      },
    });
  });

  it("falls back to the current window label", async () => {
    await attach();
    expect(invokeMock).toHaveBeenCalledWith("plugin:wallpaper|attach", {
      payload: {
        windowLabel: "current-window",
        forwardMouseInput: false,
        forwardKeyboardInput: false,
        monitor: null,
      },
    });
  });

  it("accepts an options object", async () => {
    await attach({
      windowLabel: "wallpaper",
      forwardMouseInput: true,
      forwardKeyboardInput: true,
      monitor: "DISPLAY1",
    });
    expect(invokeMock).toHaveBeenCalledWith("plugin:wallpaper|attach", {
      payload: {
        windowLabel: "wallpaper",
        forwardMouseInput: true,
        forwardKeyboardInput: true,
        monitor: "DISPLAY1",
      },
    });
  });

  it("uses the current window when options omit the label", async () => {
    await attach({
      forwardMouseInput: true,
    });
    expect(invokeMock).toHaveBeenCalledWith("plugin:wallpaper|attach", {
      payload: {
        windowLabel: "current-window",
        forwardMouseInput: true,
        forwardKeyboardInput: false,
        monitor: null,
      },
    });
  });
});

describe("detach", () => {
  it("invokes the detach command with the given window label", async () => {
    await detach("wallpaper");
    expect(invokeMock).toHaveBeenCalledWith("plugin:wallpaper|detach", {
      payload: {
        windowLabel: "wallpaper",
      },
    });
  });

  it("falls back to the current window label", async () => {
    await detach();
    expect(invokeMock).toHaveBeenCalledWith("plugin:wallpaper|detach", {
      payload: {
        windowLabel: "current-window",
      },
    });
  });
});

describe("reset", () => {
  it("invokes the reset command without a payload", async () => {
    await reset();
    expect(invokeMock).toHaveBeenCalledWith("plugin:wallpaper|reset");
  });
});

describe("pin / unpin", () => {
  it("pin invokes with the given window label", async () => {
    await pin("pin");
    expect(invokeMock).toHaveBeenCalledWith("plugin:wallpaper|pin", {
      payload: {
        windowLabel: "pin",
      },
    });
  });

  it("pin falls back to the current window label", async () => {
    await pin();
    expect(invokeMock).toHaveBeenCalledWith("plugin:wallpaper|pin", {
      payload: {
        windowLabel: "current-window",
      },
    });
  });

  it("unpin invokes with the given window label", async () => {
    await unpin("pin");
    expect(invokeMock).toHaveBeenCalledWith("plugin:wallpaper|unpin", {
      payload: {
        windowLabel: "pin",
      },
    });
  });
});

describe("capabilities", () => {
  it("invokes the capabilities command and returns the result", async () => {
    const platformCapabilities = {
      platform: "windows",
      attach: true,
      detach: true,
      reset: true,
      pin: true,
      unpin: true,
      inputForwarding: true,
      interactive: true,
      occlusion: true,
      wallpaperImage: true,
    };
    invokeMock.mockResolvedValueOnce(platformCapabilities);

    await expect(capabilities()).resolves.toEqual(platformCapabilities);
    expect(invokeMock).toHaveBeenCalledWith("plugin:wallpaper|capabilities");
  });
});

describe("state queries", () => {
  it("isAttached invokes with the given window label", async () => {
    invokeMock.mockResolvedValueOnce(true);
    await expect(isAttached("wallpaper")).resolves.toBe(true);
    expect(invokeMock).toHaveBeenCalledWith("plugin:wallpaper|is_attached", {
      payload: {
        windowLabel: "wallpaper",
      },
    });
  });

  it("isPinned falls back to the current window label", async () => {
    invokeMock.mockResolvedValueOnce(false);
    await expect(isPinned()).resolves.toBe(false);
    expect(invokeMock).toHaveBeenCalledWith("plugin:wallpaper|is_pinned", {
      payload: {
        windowLabel: "current-window",
      },
    });
  });
});

describe("setInteractive", () => {
  it("invokes with the interactive flag and window label", async () => {
    await setInteractive(true, "wallpaper");
    expect(invokeMock).toHaveBeenCalledWith("plugin:wallpaper|set_interactive", {
      payload: {
        windowLabel: "wallpaper",
        interactive: true,
      },
    });
  });

  it("falls back to the current window label", async () => {
    await setInteractive(false);
    expect(invokeMock).toHaveBeenCalledWith("plugin:wallpaper|set_interactive", {
      payload: {
        windowLabel: "current-window",
        interactive: false,
      },
    });
  });
});

describe("occlusion monitor", () => {
  it("start and stop invoke their commands", async () => {
    await startOcclusionMonitor();
    expect(invokeMock).toHaveBeenCalledWith("plugin:wallpaper|start_occlusion_monitor");

    await stopOcclusionMonitor();
    expect(invokeMock).toHaveBeenCalledWith("plugin:wallpaper|stop_occlusion_monitor");
  });
});

describe("wallpaper image", () => {
  it("setWallpaperImage invokes with the path", async () => {
    await setWallpaperImage("C:/images/bg.png");
    expect(invokeMock).toHaveBeenCalledWith("plugin:wallpaper|set_wallpaper_image", {
      payload: {
        path: "C:/images/bg.png",
      },
    });
  });

  it("getWallpaperImage returns the current image path", async () => {
    invokeMock.mockResolvedValueOnce("C:/images/current.png");
    await expect(getWallpaperImage()).resolves.toBe("C:/images/current.png");
    expect(invokeMock).toHaveBeenCalledWith("plugin:wallpaper|get_wallpaper_image");
  });
});

describe("event helpers", () => {
  it("onAttached subscribes to the attached event and unwraps the payload", async () => {
    const handler = vi.fn();
    await onAttached(handler);
    expect(listenMock).toHaveBeenCalledWith(WallpaperEvent.Attached, expect.any(Function));

    const callback = listenMock.mock.calls[0]?.[1] as (event: unknown) => void;
    callback({
      payload: {
        windowLabel: "wallpaper",
      },
    });
    expect(handler).toHaveBeenCalledWith({
      windowLabel: "wallpaper",
    });
  });

  it("onReattached subscribes to the reattached event", async () => {
    await onReattached(vi.fn());
    expect(listenMock).toHaveBeenCalledWith(WallpaperEvent.Reattached, expect.any(Function));
  });

  it("onOcclusionChanged subscribes to the occlusion event and unwraps the payload", async () => {
    const handler = vi.fn();
    await onOcclusionChanged(handler);
    expect(listenMock).toHaveBeenCalledWith(WallpaperEvent.Occlusion, expect.any(Function));

    const callback = listenMock.mock.calls[0]?.[1] as (event: unknown) => void;
    callback({
      payload: {
        windowLabel: "wallpaper",
        occluded: true,
      },
    });
    expect(handler).toHaveBeenCalledWith({
      windowLabel: "wallpaper",
      occluded: true,
    });
  });
});
