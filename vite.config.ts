import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import UnoCSS from 'unocss/vite'
import { presetIcons, presetUno } from 'unocss'
import transformerDirectives from '@unocss/transformer-directives'


// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    vue(),
    UnoCSS({
      presets: [presetIcons(), presetUno()],
      transformers: [
        transformerDirectives(),
      ],
      shortcuts: [
        ['btn', 'px-4 py-1 rounded inline-block bg-slate-700 text-white cursor-pointer !outline-none hover:bg-slate-800 disabled:cursor-default disabled:bg-gray-600 disabled:opacity-50'],
        ['icon-btn', 'hover:bg-gray-400 h-4 w-4 flex items-center justify-center rd-full inline-block cursor-pointer select-none opacity-75 transition duration-200 ease-in-out hover:opacity-100 hover:text-slate-600'],
      ],
    })
  ],

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  // prevent vite from obscuring rust errors
  clearScreen: false,
  // tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
  },
  // to make use of `TAURI_DEBUG` and other env variables
  // https://tauri.studio/v1/api/config#buildconfig.beforedevcommand
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    // Tauri supports es2021
    target: process.env.TAURI_PLATFORM == "windows" ? "chrome105" : "safari13",
    // don't minify for debug builds
    minify: !process.env.TAURI_DEBUG ? "esbuild" : false,
    // produce sourcemaps for debug builds
    sourcemap: !!process.env.TAURI_DEBUG,
  },
});
