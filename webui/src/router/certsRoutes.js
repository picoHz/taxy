const certsRoutes = [
    {
        path: "",
        name: "Server Certificate List",
        component: () => import(/* webpackChunkName: "certs" */ "@/views/Certificates.vue"),
        meta: {
            breadcrumb() {
                return [
                    {
                        trName: "server_certs.server_certs",
                        disabled: false,
                        to: { path: "/server_certs" },
                    },
                ];
            },
        },
    },
    {
        path: ":id",
        name: "Server Certificate Info",
        component: () => import(/* webpackChunkName: "cert_info" */ "@/views/CertInfo.vue"),
        meta: {
            breadcrumb(route) {
                return [
                    {
                        trName: 'server_certs.server_certs',
                        disabled: false,
                        to: { path: '/server_certs' },
                    },
                    {
                        title: route.params.id,
                        disabled: true,
                    },
                ];
            },
        },
    },
    {
        path: "acme/:id",
        name: "ACME Info",
        component: () => import(/* webpackChunkName: "acme_info" */ "@/views/AcmeInfo.vue"),
        meta: {
            breadcrumb(route) {
                return [
                    {
                        trName: 'server_certs.server_certs',
                        disabled: false,
                        to: { path: '/server_certs' },
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

export default certsRoutes;

