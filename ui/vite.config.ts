import vue from '@vitejs/plugin-vue'
import UnoCSS from 'unocss/vite'
import { defineConfig } from 'vitest/config'

export default defineConfig({ base: './', plugins: [vue(), UnoCSS()], build: { sourcemap: false }, test: { environment: 'jsdom', clearMocks: true, restoreMocks: true } })
