import { createRouter, createWebHistory } from 'vue-router';

const routes = [
  {
    path: '/',
    component: () => import('./views/index.vue'),
  },
  {
    path: '/settings',
    component: () => import('./views/settings.vue'),
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

export default router;