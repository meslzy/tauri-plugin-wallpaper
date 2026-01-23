import { defineConfig } from "tsdown";

export default defineConfig({
  entry: "lib/main.ts",
  outDir: "dist",
  format: "esm",
  dts: true,
  clean: true,
  sourcemap: true,
});
