import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  server: {
    port: 5173,
    proxy: {
      "/api": {
        // dev: if you run gateway locally use 8000; if you run user-service directly use 8001
        target: "http://localhost:8000",
        changeOrigin: true
      }
    }
  }
});

