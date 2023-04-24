const certsRoutes = [
    {
        path: "",
        name: "Certificate List",
        component: () => import(/* webpackChunkName: "certs" */ "@/views/Certificates.vue"),
        meta: {
            breadcrumb() {
                return [
                    {
                        trName: "certs.certs",
                        disabled: false,
                        to: { path: "/certs" },
                    },
                ];
            },
        },
    },
    {
        path: ":id",
        name: "Certificate Info",
        component: () => import(/* webpackChunkName: "cert_info" */ "@/views/CertInfo.vue"),
        meta: {
            breadcrumb(route) {
                return [
                    {
                        trName: 'certs.certs',
                        disabled: false,
                        to: { path: '/certs' },
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

