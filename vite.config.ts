import { resolve } from 'node:path';
import { defineConfig } from 'vite';

export default defineConfig({
  root: 'site',
  publicDir: 'public',
  build: {
    outDir: '../dist/site',
    emptyOutDir: true,
    target: 'es2022',
    cssCodeSplit: true,
    rollupOptions: {
      input: {
        index: resolve(process.cwd(), 'site/index.html'),
        privacy: resolve(process.cwd(), 'site/privacy/index.html'),
        terms: resolve(process.cwd(), 'site/terms/index.html'),
      },
    },
  },
});
