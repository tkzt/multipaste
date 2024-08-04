import { defineConfig, presetUno, transformerDirectives, transformerVariantGroup } from 'unocss'
import presetIcons from '@unocss/preset-icons'

export default defineConfig({
  transformers: [
    transformerDirectives(),
    transformerVariantGroup(),
  ],
  presets: [presetUno({
    dark: 'media',
  }), presetIcons()],
  shortcuts: [
    ['card', 'rd-lg hover:(bg-white/20 dark:bg-white/20) bg-white/30 p-2 dark:(bg-white/12 text-gray-100)'],
  ],
})
