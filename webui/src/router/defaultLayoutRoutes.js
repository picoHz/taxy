import portsRoutes from "./portsRoutes";
import certsRoutes from "./certsRoutes";

const defaultLayoutRoutes = [
    {
        path: "ports",
        name: "Ports",
        alias: "/",
        component: () => import(/* webpackChunkName: "ports" */ "@/layouts/default/View.vue"),
        children: portsRoutes,
    },
    {
        path: "keyring",
        name: "Keyring",
        component: () => import(/* webpackChunkName: "ports" */ "@/layouts/default/View.vue"),
        children: certsRoutes,
    },
];

export default defaultLayoutRoutes;