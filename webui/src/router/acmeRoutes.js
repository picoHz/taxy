const certsRoutes = [
    {
        path: "",
        name: "ACME List",
        component: () => import(/* webpackChunkName: "certs" */ "@/views/Certificates.vue"),
        meta: {
            breadcrumb() {
                return [
                    {
                        trName: "keyring.keyring",
                        disabled: false,
                        to: { path: "/acme" },
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
                        trName: 'keyring.keyring',
                        disabled: false,
                        to: { path: '/acme' },
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

