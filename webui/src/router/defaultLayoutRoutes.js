import portsRoutes from "./portsRoutes";
import sitesRoutes from "./sitesRoutes";
import certsRoutes from "./certsRoutes";
import acmeRoutes from "./acmeRoutes";

const defaultLayoutRoutes = [
    {
        path: "ports",
        name: "Ports",
        alias: "/",
        component: () => import(/* webpackChunkName: "ports" */ "@/layouts/default/View.vue"),
        children: portsRoutes,
    },
    {
        path: "sites",
        name: "Sites",
        component: () => import(/* webpackChunkName: "sites" */ "@/layouts/default/View.vue"),
        children: sitesRoutes,
    },
    {
        path: "certs",
        name: "Certificates",
        component: () => import(/* webpackChunkName: "certs" */ "@/layouts/default/View.vue"),
        children: certsRoutes,
    },
    {
        path: "acme",
        name: "ACME",
        component: () => import(/* webpackChunkName: "acme" */ "@/layouts/default/View.vue"),
        children: acmeRoutes,
    },
];

export default defaultLayoutRoutes;