import { defineConfig } from "@farmfe/core";
import path from "path";

export default defineConfig({
  plugins: ["@farmfe/plugin-react"],
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
