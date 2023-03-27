/**
 * plugins/index.js
 *
 * Automatically included in `./src/main.js`
 */

// Plugins
import { loadFonts } from './webfontloader'
import vuetify from './vuetify'
import router from '../router'
import { createPinia } from 'pinia'
import { createI18n } from 'vue-i18n'
import en_US from '@/i18n/en_US.js'

export function registerPlugins(app) {
  loadFonts()
  app
    .use(vuetify)
    .use(router)
    .use(createPinia())
    .use(createI18n(
      {
        legacy: false,
        fallbackLocale: 'en-US',
        messages: { 'en-US': en_US }
      }
    ))
}
