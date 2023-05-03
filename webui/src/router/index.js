import { createRouter, createWebHistory } from "vue-router";
import defaultLayoutRoutes from "./defaultLayoutRoutes";

const routes = [
  {
    path: "/",
    component: () => import("@/layouts/default/Default.vue"),
    children: defaultLayoutRoutes,
  },
  {
    path: "/login",
    name: "Login",
    component: () => import(/* webpackChunkName: "login" */ "@/layouts/default/Login.vue"),
  },
];

const router = createRouter({
  history: createWebHistory(process.env.BASE_URL),
  routes,
});

router.beforeEach(async (to, from) => {
  const isAuthenticated = localStorage.getItem("token");
  if (
    !isAuthenticated && to.name !== 'Login'
  ) {
    return { name: 'Login' }
  }
})

export default router;