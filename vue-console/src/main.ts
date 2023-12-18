import { createApp } from 'vue'
import './style.css'
import App from './App.vue'
import router from './router'
import store from './store'
import ElementPlus from 'element-plus'
import 'element-plus/dist/index.css'

createApp(App)
    .use(ElementPlus)
    .use(router)
    .use(store)
    .mount('#app')
