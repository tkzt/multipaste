import { defineConfig, presetUno, transformerDirectives, transformerVariantGroup } from 'unocss'
import presetIcons from '@unocss/preset-icons'

export default defineConfig({
  transformers: [
    transformerDirectives(),
    transformerVariantGroup(),
  ],
  presets: [presetUno(), presetIcons()],
})
