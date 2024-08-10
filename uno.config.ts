import { defineConfig, presetUno, transformerDirectives, transformerVariantGroup } from 'unocss'

export default defineConfig({
  transformers: [
    transformerDirectives(),
    transformerVariantGroup(),
  ],
  presets: [presetUno({
    dark: 'media',
  })],
  shortcuts: [
    ['card', 'rd-lg hover:(bg-white/20 dark:bg-white/20) bg-white/30 p-2 dark:(bg-white/12 text-gray-100)'],
    ['btn', 'block w-6 h-6 flex justify-center items-center rd-50% cursor-pointer hover:bg-[rgba(0,0,0,.05)] important:active:bg-[rgba(0,0,0,.1)] active:scale-105 dark:hover:bg-[rgba(255,255,255,.25)] important:dark:active:bg-[rgba(255,255,255,.3)]'],
    ['active-btn', ''],
  ],
})
