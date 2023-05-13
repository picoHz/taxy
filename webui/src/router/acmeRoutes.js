const acmeRoutes = [
    {
        path: "",
        name: "ACME List",
        component: () => import(/* webpackChunkName: "certs" */ "@/views/AcmeList.vue"),
        meta: {
            breadcrumb() {
                return [
                    {
                        trName: "acme.acme",
                        disabled: false,
                        to: { path: "/acme" },
                    },
                ];
            },
        },
    },
    {
        path: ":id",
        name: "ACME Info",
        component: () => import(/* webpackChunkName: "acme_info" */ "@/views/AcmeInfo.vue"),
        meta: {
            breadcrumb(route) {
                return [
                    {
                        trName: 'acme.acme',
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

export default acmeRoutes;

