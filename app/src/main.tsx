import "./main.css";

import { invoke } from "@tauri-apps/api/core";
import { availableMonitors } from "@tauri-apps/api/window";
import React, { useEffect, useState } from "react";
import ReactDOM from "react-dom/client";
import wallpaper, {
  type Capabilities,
  onAttached,
  onDetached,
  onOcclusionChanged,
  onPinned,
  onReattached,
  onUnpinned,
  pin as pinWindow,
  unpin as unpinWindow,
} from "tauri-plugin-wallpaper";

const App = () => {
  const [capabilities, setCapabilities] = useState<Capabilities | null>(null);
  const [status, setStatus] = useState("");
  const [lastEvent, setLastEvent] = useState("");
  const [monitors, setMonitors] = useState<string[]>([]);
  const [monitor, setMonitor] = useState("");
  const [forwardMouse, setForwardMouse] = useState(false);
  const [forwardKeyboard, setForwardKeyboard] = useState(false);
  const [interactive, setInteractive] = useState(false);
  const [monitoringOcclusion, setMonitoringOcclusion] = useState(false);
  const [imagePath, setImagePath] = useState("");

  useEffect(() => {
    wallpaper
      .capabilities()
      .then(setCapabilities)
      .catch((error) => setStatus(`capabilities failed: ${error}`));

    availableMonitors()
      .then((list) => setMonitors(list.map((m) => m.name ?? "").filter(Boolean)))
      .catch(() => {});

    const unlisteners = [
      onAttached((p) => setLastEvent(`attached: ${p.windowLabel}`)),
      onDetached((p) => setLastEvent(`detached: ${p.windowLabel}`)),
      onPinned((p) => setLastEvent(`pinned: ${p.windowLabel}`)),
      onUnpinned((p) => setLastEvent(`unpinned: ${p.windowLabel}`)),
      onReattached((p) => setLastEvent(`reattached after explorer restart: ${p.windowLabel}`)),
      onOcclusionChanged((p) => setLastEvent(`occlusion: ${p.windowLabel} is ${p.occluded ? "covered" : "visible"}`)),
    ];

    return () => {
      for (const unlisten of unlisteners) {
        unlisten.then((fn) => fn()).catch(() => {});
      }
    };
  }, []);

  const run = async (name: string, task: () => Promise<unknown>) => {
    try {
      const result = await task();
      setStatus(result === undefined ? `${name}: ok` : `${name}: ${result}`);
    } catch (error) {
      setStatus(`${name} failed: ${error}`);
    }
  };

  const attachWallpaper = () =>
    run("attach", () =>
      wallpaper.attach({
        windowLabel: "wallpaper",
        forwardMouseInput: forwardMouse,
        forwardKeyboardInput: forwardKeyboard,
        monitor: monitor || undefined,
      }),
    );
  const detachWallpaper = () => run("detach", () => wallpaper.detach("wallpaper"));
  const resetWallpaper = () => run("reset", () => wallpaper.reset());
  const showWallpaper = () => run("show wallpaper", () => invoke("show_wallpaper"));
  const hideWallpaper = () => run("hide wallpaper", () => invoke("hide_wallpaper"));

  const toggleInteractive = () => {
    const next = !interactive;
    run(`setInteractive(${next})`, () => wallpaper.setInteractive(next, "wallpaper")).then(() => setInteractive(next));
  };

  const toggleOcclusionMonitor = () => {
    if (monitoringOcclusion) {
      run("stop occlusion monitor", () => wallpaper.stopOcclusionMonitor()).then(() => setMonitoringOcclusion(false));
    } else {
      run("start occlusion monitor", () => wallpaper.startOcclusionMonitor()).then(() => setMonitoringOcclusion(true));
    }
  };

  const queryState = () =>
    run("state", async () => {
      const [attached, pinned] = await Promise.all([
        wallpaper.isAttached("wallpaper"),
        wallpaper.isPinned("pin"),
      ]);
      return `wallpaper attached=${attached}, pin pinned=${pinned}`;
    });

  const applyWallpaperImage = () => run("set wallpaper image", () => wallpaper.setWallpaperImage(imagePath));
  const readWallpaperImage = () => run("get wallpaper image", () => wallpaper.getWallpaperImage());

  const pin = () => run("pin", () => pinWindow("pin"));
  const unpin = () => run("unpin", () => unpinWindow("pin"));
  const showPin = () => run("show pin", () => invoke("show_pin"));
  const hidePin = () => run("hide pin", () => invoke("hide_pin"));

  const quit = async () => {
    await invoke("quit");
  };

  const button = (label: string, onClick: () => void, background: string, supported: boolean) => (
    <button
      disabled={!supported}
      onClick={onClick}
      style={{
        background,
        cursor: supported ? "pointer" : "not-allowed",
        opacity: supported ? 1 : 0.4,
      }}
      title={supported ? undefined : "Not supported on this platform"}
      type="button"
    >
      {label}
    </button>
  );

  return (
    <>
      <section>
        <h3>Platform</h3>
        {capabilities ? (
          <p>
            {capabilities.platform} — wallpaper: {capabilities.attach ? "yes" : "no"}, pin:{" "}
            {capabilities.pin ? "yes" : "no"}, input forwarding: {capabilities.inputForwarding ? "yes" : "no"},
            occlusion: {capabilities.occlusion ? "yes" : "no"}, wallpaper image:{" "}
            {capabilities.wallpaperImage ? "yes" : "no"}
          </p>
        ) : (
          <p>loading capabilities…</p>
        )}
      </section>

      <section>
        <h3>Wallpaper</h3>
        <div>
          <label>
            <input
              checked={forwardMouse}
              disabled={!capabilities?.inputForwarding}
              onChange={(e) => setForwardMouse(e.target.checked)}
              type="checkbox"
            />
            forward mouse
          </label>
          <label>
            <input
              checked={forwardKeyboard}
              disabled={!capabilities?.inputForwarding}
              onChange={(e) => setForwardKeyboard(e.target.checked)}
              type="checkbox"
            />
            forward keyboard
          </label>
          <select
            onChange={(e) => setMonitor(e.target.value)}
            value={monitor}
          >
            <option value="">current bounds</option>
            {monitors.map((name) => (
              <option
                key={name}
                value={name}
              >
                {name}
              </option>
            ))}
          </select>
        </div>
        <div>
          {button("Attach", attachWallpaper, "#34a0a6", capabilities?.attach ?? false)}
          {button("Detach", detachWallpaper, "#ad1d02", capabilities?.detach ?? false)}
          {button("Reset", resetWallpaper, "#acb06b", capabilities?.reset ?? false)}
        </div>
        <div>
          {button(
            interactive ? "Passive" : "Interactive",
            toggleInteractive,
            "#8862c9",
            capabilities?.interactive ?? false,
          )}
          {button(
            monitoringOcclusion ? "Stop occlusion" : "Watch occlusion",
            toggleOcclusionMonitor,
            "#c9628f",
            capabilities?.occlusion ?? false,
          )}
          {button("Query state", queryState, "#5f8a5c", true)}
        </div>
        <div>
          {button("Show", showWallpaper, "#34a0a6", true)}
          {button("Hide", hideWallpaper, "#ad1d02", true)}
        </div>
      </section>

      <section>
        <h3>Wallpaper image</h3>
        <div>
          <input
            onChange={(e) => setImagePath(e.target.value)}
            placeholder="C:\path\to\image.png"
            type="text"
            value={imagePath}
          />
        </div>
        <div>
          {button(
            "Set image",
            applyWallpaperImage,
            "#34a0a6",
            (capabilities?.wallpaperImage ?? false) && imagePath.length > 0,
          )}
          {button("Get image", readWallpaperImage, "#5f8a5c", capabilities?.wallpaperImage ?? false)}
        </div>
      </section>

      <section>
        <h3>Pin</h3>
        <div>
          {button("Pin", pin, "#667eea", capabilities?.pin ?? false)}
          {button("Unpin", unpin, "#764ba2", capabilities?.unpin ?? false)}
        </div>
        <div>
          {button("Show", showPin, "#34a0a6", true)}
          {button("Hide", hidePin, "#ad1d02", true)}
        </div>
      </section>

      <section>
        <button
          onClick={quit}
          style={{
            background: "#333",
          }}
          type="button"
        >
          Quit
        </button>
      </section>

      {(status || lastEvent) && (
        <section>
          {status && <p>{status}</p>}
          {lastEvent && <p>event: {lastEvent}</p>}
        </section>
      )}
    </>
  );
};

const rootElement = document.getElementById("root");

if (!rootElement) throw new Error("Failed to find the root element");

const root = ReactDOM.createRoot(rootElement);

root.render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
