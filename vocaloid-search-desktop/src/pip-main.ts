import { createApp } from 'vue'
import { i18n } from './i18n'
import PipApp from './PipApp.vue'
import './pip-style.css'

createApp(PipApp)
  .use(i18n)
  .mount('#app')
