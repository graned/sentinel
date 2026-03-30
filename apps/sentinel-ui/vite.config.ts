import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { VitePWA } from 'vite-plugin-pwa';

// https://vite.dev/config/
export default defineConfig({
  resolve: {
    // Required for file: / symlinked local packages (e.g. @sentinel/auth-react).
    // Without this Vite follows the symlink to the real path and can't find the
    // package's peer deps (zustand, react-query, etc.) which live in /app/node_modules.
    preserveSymlinks: true,
  },
  plugins: [
    react(),
    VitePWA({
      registerType: 'autoUpdate',
      includeAssets: ['icons/favicon.svg', 'icons/sentinel-logo.svg', 'icons/pwa-192.svg', 'icons/pwa-512.svg'],
      manifest: {
        name: 'Sentinel Admin',
        short_name: 'Sentinel',
        description: 'Sentinel Auth Management Console',
        theme_color: '#0f172a',
        background_color: '#0B1120',
        display: 'standalone',
        scope: '/',
        start_url: '/',
        icons: [
          { src: '/icons/pwa-192.svg', sizes: '192x192', type: 'image/svg+xml' },
          {
            src: '/icons/pwa-512.svg',
            sizes: '512x512',
            type: 'image/svg+xml',
            purpose: 'any maskable',
          },
        ],
      },
      workbox: {
        globPatterns: ['**/*.{js,css,html,svg,png,ico}'],
      },
    }),
  ],
  server: {
    host: true, // bind to 0.0.0.0 so Docker exposes the port to the host
    port: 3000,
    proxy: {
      '/v1': {
        target: process.env.VITE_API_URL ?? 'http://localhost:8080',
        changeOrigin: true,
      },
    },
  },
});
