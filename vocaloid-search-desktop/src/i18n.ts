import { createI18n } from 'vue-i18n'
import type { Locale } from './stores/locale'
import ja from './locales/ja.json'
import en from './locales/en.json'
import zhTW from './locales/zh-TW.json'

const messages = {
  ja,
  en,
  'zh-TW': zhTW,
}

const savedLocale = localStorage.getItem('locale') as Locale || 'zh-TW'

export const i18n = createI18n({
  legacy: false,
  locale: savedLocale,
  fallbackLocale: 'ja',
  messages,
})
