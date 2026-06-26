import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import wasm from "vite-plugin-wasm";
import { wasmPackPlugin } from "./vite-wasm-pack-plugin";

export default defineConfig({
  plugins: [react(), wasm(), wasmPackPlugin()],
  server: {
    fs: {
      allow: [".."],
    },
  },
  optimizeDeps: {
    exclude: ["./src/wasm/editor_core.js"],
  },
});
