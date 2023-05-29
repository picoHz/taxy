const sitesRoutes = [
    {
        path: "",
        name: "Site List",
        component: () => import(/* webpackChunkName: "sites" */ "@/views/SiteList.vue"),
        meta: {
            breadcrumb() {
                return [
                    {
                        trName: "sites.sites",
                        disabled: false,
                        to: { path: "/sites" },
                    },
                ];
            },
        },
    },
    {
        path: ":id",
        name: "Site Info",
        component: () => import(/* webpackChunkName: "site_info" */ "@/views/SiteInfo.vue"),
        meta: {
            breadcrumb(route) {
                return [
                    {
                        trName: 'sites.sites',
                        disabled: false,
                        to: { path: '/sites' },
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

export default sitesRoutes;

