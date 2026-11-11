import { defineConfig, presetIcons } from 'unocss'

const icons = [
  'i-carbon-copy',
  'i-carbon-cut',
  'i-carbon-paste',
  'i-carbon-undo',
  'i-carbon-redo',
  'i-carbon-select-window',
  'i-carbon-save',
  'i-carbon-search',
  'i-carbon-folder',
  'i-carbon-keyboard',
]

export default defineConfig({
  presets: [
    presetIcons({
      scale: 1,
      extraProperties: {
        'display': 'inline-block',
        'vertical-align': 'middle',
      },
    }),
  ],
  safelist: icons,
})
