import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { createRouter, createWebHashHistory } from 'vue-router'
import { i18n } from './i18n'
import App from './App.vue'
import './style.css'

import SearchView from './views/SearchView.vue'
import HistoryView from './views/HistoryView.vue'
import ScraperView from './views/ScraperView.vue'

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: '/', name: 'search', component: SearchView },
    { path: '/history', name: 'history', component: HistoryView },
    { path: '/scraper', name: 'scraper', component: ScraperView },
  ],
})

const pinia = createPinia()

createApp(App)
  .use(pinia)
  .use(router)
  .use(i18n)
  .mount('#app')
