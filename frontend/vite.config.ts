import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import svgr from 'vite-plugin-svgr';

// https://vite.dev/config/
export default defineConfig({
  plugins: [react(), svgr()],
  server: {
    cors: {
      origin: "http://localhost:2180"
    },
    origin: "http://localhost:5173",
    fs: {
      // Allow serving files from one level up to the project root
      allow: ['../..'],
    }
  },
  build: {
    manifest: true,
    rollupOptions: {
      input: "src/main.tsx"
    },
    modulePreload: {
      polyfill: false
    }
  },
  optimizeDeps: {
    exclude: ["@ironcalc/wasm"]
  }
})
