import React from "react";

import wallpaper from "tauri-plugin-wallpaper";

import { invoke } from "@tauri-apps/api/core";
import ReactDOM from "react-dom/client";

import "./main.css";

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
    <React.Fragment>
      <div>
        <button style={{ background: "#34a0a6" }} id={"attach"} onClick={attach}>Attach</button>
        <button style={{ background: "#ad1d02" }} id={"detach"} onClick={detach}>Detach</button>
        <button style={{ background: "#acb06b" }} id={"reset"} onClick={reset}>Reset</button>
      </div>

      <div>
        <button style={{ background: "#34a0a6" }} id={"show"} onClick={show}>Show</button>
        <button style={{ background: "#ad1d02" }} id={"hide"} onClick={hide}>Hide</button>
      </div>
    </React.Fragment>
  );
};

const root = ReactDOM.createRoot(document.querySelector("#root")!);

root.render(
  <React.StrictMode>
    <App/>
  </React.StrictMode>,
);
