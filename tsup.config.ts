import { defineConfig } from "tsup";

export default defineConfig({
  clean: true,
  dts: true,
  format: "esm",
  entry: {
    main: "lib/main.ts",
  },
});
