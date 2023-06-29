import { defineConfig } from "vite";

import react from "@vitejs/plugin-react";
import tsconfigPaths from "vite-tsconfig-paths";

const development = process.env.TAURI_DEBUG == "true";
const production = process.env.TAURI_DEBUG !== "true";

export default defineConfig({
  plugins: [
    react(),
    tsconfigPaths(),
  ],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
  },
  envPrefix: [
    "VITE_",
    "TAURI_",
  ],
  build: {
    target: [ "es2021", "chrome100", "safari13" ],
    minify: production ? "esbuild" : false,
    sourcemap: development ? "inline" : false,
  },
});