// Composables
import { createRouter, createWebHistory } from 'vue-router'

const routes = [
  {
    path: '/',
    component: () => import('@/layouts/default/Default.vue'),
    children: [
      {
        path: 'ports',
        name: 'Ports',
        alias: '/',
        component: () => import(/* webpackChunkName: "ports" */ '@/views/Ports.vue'),
        children: [
          {
            path: '',
            name: 'List',
            component: () => import(/* webpackChunkName: "portsx" */ '@/views/PortList.vue'),
            meta: {
              breadcrumb() {
                return [{
                  trName: 'ports.ports',
                  disabled: false,
                  to: { path: '/' }
                }]
              }
            }
          },
          {
            path: 'new',
            name: 'New Port',
            component: () => import(/* webpackChunkName: "new-port" */ '@/views/NewPort.vue'),
            meta: {
              breadcrumb() {
                return [{
                  trName: 'ports.ports',
                  disabled: false,
                  to: { path: '/' }
                }, {
                  trName: 'ports.new_port',
                  disabled: true
                }]
              }
            }
          },
          {
            path: ':name',
            name: 'Port Status',
            component: () => import(/* webpackChunkName: "portsn" */ '@/views/PortStatus.vue'),
            meta: {
              breadcrumb(route) {
                return [{
                  trName: 'ports.ports',
                  disabled: false,
                  to: { path: '/' }
                }, {
                  title: route.params.name,
                  disabled: true
                }]
              }
            }
          }
        ]
      },
    ],
  },
]

const router = createRouter({
  history: createWebHistory(process.env.BASE_URL),
  routes,
})

export default router
