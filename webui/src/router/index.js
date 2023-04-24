import { createRouter, createWebHistory } from "vue-router";
import defaultLayoutRoutes from "./defaultLayoutRoutes";

const routes = [
  {
    path: "/",
    component: () => import("@/layouts/default/Default.vue"),
    children: defaultLayoutRoutes,
  },
];

const router = createRouter({
  history: createWebHistory(process.env.BASE_URL),
  routes,
});

export default router;