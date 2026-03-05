import { defineStore } from 'pinia'
import { ref, watch } from 'vue'

export type Locale = 'ja' | 'en' | 'zh-TW'

export const useLocaleStore = defineStore('locale', () => {
  const locale = ref<Locale>((localStorage.getItem('locale') as Locale) || 'zh-TW')

  watch(locale, (newLocale) => {
    localStorage.setItem('locale', newLocale)
  })

  function setLocale(l: Locale) {
    locale.value = l
  }

  return {
    locale,
    setLocale,
  }
})
