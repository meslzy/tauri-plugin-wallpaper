import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig(async () => ({
  plugins: [
    react(),
  ],
  build: {
    rollupOptions: {
      input: {
        main: "main.html",
        wallpaper: "wallpaper.html",
      },
    },
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ?
      {
        protocol: "ws",
        host,
        port: 1421,
      } :
      undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
}));
