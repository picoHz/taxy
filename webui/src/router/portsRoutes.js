const portsRoutes = [
    {
        path: "",
        name: "List",
        component: () => import(/* webpackChunkName: "portsx" */ "@/views/PortList.vue"),
        meta: {
            breadcrumb() {
                return [
                    {
                        trName: "ports.ports",
                        disabled: false,
                        to: { path: "/" },
                    },
                ];
            },
        },
    },
    {
        path: "new",
        name: "New Port",
        component: () => import(/* webpackChunkName: "new-port" */ "@/views/NewPort.vue"),
        meta: {
            breadcrumb() {
                return [
                    {
                        trName: "ports.ports",
                        disabled: false,
                        to: { path: "/" },
                    },
                    {
                        trName: "ports.new_port",
                        disabled: true,
                    },
                ];
            },
        },
    },
    {
        path: ":id",
        name: "Port Status",
        component: () => import(/* webpackChunkName: "portsn" */ "@/views/PortStatus.vue"),
        meta: {
            breadcrumb(route) {
                return [
                    {
                        trName: "ports.ports",
                        disabled: false,
                        to: { path: "/" },
                    },
                    {
                        title: route.params.id,
                        disabled: true,
                    },
                ];
            },
        },
    },
];

export default portsRoutes;
