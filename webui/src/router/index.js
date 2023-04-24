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
        component: () => import(/* webpackChunkName: "ports" */ '@/layouts/default/View.vue'),
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
      {
        path: 'certs',
        name: 'Certificates',
        component: () => import(/* webpackChunkName: "ports" */ '@/layouts/default/View.vue'),
        children: [
          {
            path: '',
            name: 'Certificate List',
            component: () => import(/* webpackChunkName: "certs" */ '@/views/Certificates.vue'),
            meta: {
              breadcrumb() {
                return [{
                  trName: 'certs.certs',
                  disabled: false,
                  to: { path: '/certs' }
                }]
              }
            }
          },
          {
            path: ':id',
            name: 'Certificate Info',
            component: () => import(/* webpackChunkName: "cert_info" */ '@/views/CertInfo.vue'),
            meta: {
              breadcrumb(route) {
                return [{
                  trName: 'certs.certs',
                  disabled: false,
                  to: { path: '/certs' }
                }, {
                  title: route.params.id,
                  disabled: true
                }]
              }
            }
          }
        ]
      }
    ],
  },
]

const router = createRouter({
  history: createWebHistory(process.env.BASE_URL),
  routes,
})

export default router
