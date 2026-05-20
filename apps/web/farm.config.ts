import { defineConfig } from "@farmfe/core";
import postcss from "@farmfe/js-plugin-postcss";
import react from "@farmfe/plugin-react";
import path from "path";

export default defineConfig({
  plugins: [react(), postcss()],
  compilation: {
    resolve: {
      alias: {
        "@/": path.join(process.cwd(), "src"),
      },
    },
  },
  server: {
    port: 4000,
    proxy: {
      "/api": {
        target: "http://localhost:1234",
        changeOrigin: true,
      },
    },
  },
});
