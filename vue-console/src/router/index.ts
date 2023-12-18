
import { createRouter, createWebHistory, RouteRecordRaw } from 'vue-router';
import Login from '../views/Login.vue';
 
const routes: Array<RouteRecordRaw> = [
  {
    path: '/',
    name: 'Login',
    component: Login,
  },
];
 
const router = createRouter({
  history: createWebHistory(),
  routes,
});
 
export default router;