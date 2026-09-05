import { execFileSync } from 'node:child_process';
import { resolve } from 'node:path';
import { defineConfig } from 'vite';

const buildId = process.env.BUILD_ID ?? execFileSync('git', ['rev-parse', '--short=12', 'HEAD']).toString().trim();

export default defineConfig({
  root: 'site',
  publicDir: 'public',
  plugins: [{
    name: 'build-id',
    transformIndexHtml(html) {
      return html.replaceAll('%BUILD_ID%', buildId);
    },
  }],
  build: {
    outDir: '../dist/site',
    emptyOutDir: true,
    target: 'es2022',
    cssCodeSplit: true,
    rollupOptions: {
      input: {
        index: resolve(process.cwd(), 'site/index.html'),
        demo: resolve(process.cwd(), 'site/demo/index.html'),
        privacy: resolve(process.cwd(), 'site/privacy/index.html'),
        terms: resolve(process.cwd(), 'site/terms/index.html'),
        notFound: resolve(process.cwd(), 'site/404.html'),
      },
    },
  },
});
