import "./main.css";

import { invoke } from "@tauri-apps/api/core";
import React from "react";
import ReactDOM from "react-dom/client";
import wallpaper, { pin as pinWindow, unpin as unpinWindow } from "tauri-plugin-wallpaper";

const App = () => {
  const attachWallpaper = async () => {
    const response = await wallpaper.attach("wallpaper");
    console.log("attach wallpaper", response);
  };

  const detachWallpaper = async () => {
    const response = await wallpaper.detach("wallpaper");
    console.log("detach wallpaper", response);
  };

  const resetWallpaper = async () => {
    const response = await wallpaper.reset();
    console.log("reset wallpaper", response);
  };

  const showWallpaper = async () => {
    const response = await invoke("show_wallpaper");
    console.log("show wallpaper", response);
  };

  const hideWallpaper = async () => {
    const response = await invoke("hide_wallpaper");
    console.log("hide wallpaper", response);
  };

  const pin = async () => {
    const response = await pinWindow("pin");
    console.log("pin", response);
  };

  const unpin = async () => {
    const response = await unpinWindow("pin");
    console.log("unpin", response);
  };

  const showPin = async () => {
    const response = await invoke("show_pin");
    console.log("show pin", response);
  };

  const hidePin = async () => {
    const response = await invoke("hide_pin");
    console.log("hide pin", response);
  };

  const quit = async () => {
    await invoke("quit");
  };

  return (
    <>
      <section>
        <h3>Wallpaper</h3>
        <div>
          <button
            onClick={attachWallpaper}
            style={{
              background: "#34a0a6",
            }}
            type="button"
          >
            Attach
          </button>
          <button
            onClick={detachWallpaper}
            style={{
              background: "#ad1d02",
            }}
            type="button"
          >
            Detach
          </button>
          <button
            onClick={resetWallpaper}
            style={{
              background: "#acb06b",
            }}
            type="button"
          >
            Reset
          </button>
        </div>
        <div>
          <button
            onClick={showWallpaper}
            style={{
              background: "#34a0a6",
            }}
            type="button"
          >
            Show
          </button>
          <button
            onClick={hideWallpaper}
            style={{
              background: "#ad1d02",
            }}
            type="button"
          >
            Hide
          </button>
        </div>
      </section>

      <section>
        <h3>Pin</h3>
        <div>
          <button
            onClick={pin}
            style={{
              background: "#667eea",
            }}
            type="button"
          >
            Pin
          </button>
          <button
            onClick={unpin}
            style={{
              background: "#764ba2",
            }}
            type="button"
          >
            Unpin
          </button>
        </div>
        <div>
          <button
            onClick={showPin}
            style={{
              background: "#34a0a6",
            }}
            type="button"
          >
            Show
          </button>
          <button
            onClick={hidePin}
            style={{
              background: "#ad1d02",
            }}
            type="button"
          >
            Hide
          </button>
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
