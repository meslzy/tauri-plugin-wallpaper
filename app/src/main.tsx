import "./main.css";

import { invoke } from "@tauri-apps/api/core";
import React from "react";
import ReactDOM from "react-dom/client";
import wallpaper from "tauri-plugin-wallpaper";

const App = () => {
  const attach = async () => {
    const response = await wallpaper.attach("wallpaper");
    console.log("attach", response);
  };

  const detach = async () => {
    const response = await wallpaper.detach("wallpaper");
    console.log("detach", response);
  };

  const reset = async () => {
    const response = await wallpaper.reset();
    console.log("reset", response);
  };

  const show = async () => {
    const response = await invoke("show");
    console.log("show", response);
  };

  const hide = async () => {
    const response = await invoke("hide");
    console.log("hide", response);
  };

  return (
    <>
      <div>
        <button
          id={"attach"}
          onClick={attach}
          style={{
            background: "#34a0a6",
          }}
          type="button"
        >
          Attach
        </button>
        <button
          id={"detach"}
          onClick={detach}
          style={{
            background: "#ad1d02",
          }}
          type="button"
        >
          Detach
        </button>
        <button
          id={"reset"}
          onClick={reset}
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
          id={"show"}
          onClick={show}
          style={{
            background: "#34a0a6",
          }}
          type="button"
        >
          Show
        </button>
        <button
          id={"hide"}
          onClick={hide}
          style={{
            background: "#ad1d02",
          }}
          type="button"
        >
          Hide
        </button>
      </div>
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
