/**
 * Vite Configuration - Optimized for AnyCowork
 */

import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";

export default defineConfig({
  plugins: [
    react({
      // Enable Fast Refresh
      fastRefresh: true,
      // Remove dev-only code in production
      babel: {
        plugins: [
          // process.env.NODE_ENV === "production" && [
          //   "transform-remove-console",
          //   { exclude: ["error", "warn"] },
          // ],
        ].filter(Boolean),
      },
    }),
  ],

  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./"),
    },
  },

  server: {
    port: 5173,
    strictPort: true,
    host: true, // Listen on all addresses
    // Proxy not strictly needed if we access via backend, but helpful for direct dev
    proxy: {
      "/api": {
        target: process.env.VITE_API_URL || "http://localhost:8080",
        changeOrigin: true,
      },
    },
    hmr: {
      clientPort: 5173,
    },
    watch: {
      // Exclude directories that shouldn't be watched
      ignored: [
        "**/node_modules/**",
        "**/src-tauri/target/**",
        "**/dist/**",
        "**/.git/**",
      ],
    },
  },

  build: {
    outDir: "dist",
    emptyOutDir: true,
    sourcemap: false,
    minify: "esbuild", // Faster than terser
    target: "esnext", // Modern browsers only

    // Optimize chunk sizes
    chunkSizeWarningLimit: 1000,

    rollupOptions: {
      output: {
        // Optimize manual chunks for better caching
        // Optimize manual chunks for better caching
        // manualChunks: (id) => {
        //   // Vendor chunk for core libraries
        //   if (id.includes("node_modules")) {
        //     if (id.includes("react") || id.includes("react-dom")) {
        //       return "vendor-react";
        //     }
        //     if (id.includes("react-router")) {
        //       return "vendor-router";
        //     }
        //     if (id.includes("@tanstack/react-query")) {
        //       return "vendor-query";
        //     }
        //     if (id.includes("lucide-react")) {
        //       return "vendor-icons";
        //     }
        //     if (id.includes("@radix-ui")) {
        //       return "vendor-ui";
        //     }
        //     // All other node_modules in one chunk
        //     return "vendor";
        //   }
        // },

        // Optimize filenames for better caching
        chunkFileNames: "assets/[name]-[hash].js",
        entryFileNames: "assets/[name]-[hash].js",
        assetFileNames: "assets/[name]-[hash][extname]",
      },
    },

    // Enable CSS code splitting
    cssCodeSplit: true,
  },

  // Optimize dependencies
  optimizeDeps: {
    include: [
      "react",
      "react-dom",
      "react-router-dom",
      "@tanstack/react-query",
    ],
  },
});
