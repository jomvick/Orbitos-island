import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      "@agentos/shared-schema": path.resolve(
        __dirname,
        "../../packages/shared-schema/src"
      ),
    },
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    fs: {
      allow: ["..", "../.."],
    },
  },
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    target: process.env.TAURI_PLATFORM === "windows" ? "chrome105" : "safari13",
    minify: !process.env.TAURI_DEBUG ? "esbuild" : false,
    sourcemap: !!process.env.TAURI_DEBUG,
  },
});
