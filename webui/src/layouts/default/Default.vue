<template>
  <v-app>
    <v-layout>
      <v-navigation-drawer v-model="drawer">
        <v-list>
          <v-list-item prepend-icon="mdi-network" title="Ports" :to="{ path: '/ports' }"></v-list-item>
          <v-list-item prepend-icon="mdi-certificate" title="Certificates" :to="{ path: '/certs' }"></v-list-item>
          <v-list-item prepend-icon="mdi-cloud-lock" title="ACME" :to="{ path: '/acme' }"></v-list-item>
          <v-list-item prepend-icon="mdi-code-json" append-icon="mdi-open-in-new" title="Swagger" href="/swagger-ui"
            target="_blank"></v-list-item>
        </v-list>
        <template v-slot:append>
          <div class="pa-2">
            <v-btn @click="logout" block>
              Logout
            </v-btn>
          </div>
        </template>
      </v-navigation-drawer>

      <v-main>
        <v-toolbar color="primary" dark extended flat>
          <v-app-bar-nav-icon @click="drawer = !drawer"></v-app-bar-nav-icon>
        </v-toolbar>

        <v-card class="mx-auto" max-width="700" style="margin-top: -64px;">
          <v-breadcrumbs :items="breadcrumbs">
            <template v-slot:divider>
              <v-icon icon="mdi-chevron-right"></v-icon>
            </template>
            <template v-slot:title="{ item }">
              <span v-if="item.trName" class="text-h6">{{ t(item.trName, item.trData) }}</span>
              <span v-else class="text-h6">{{ item.title }}</span>
            </template>
          </v-breadcrumbs>
          <router-view />
        </v-card>
      </v-main>
    </v-layout>
  </v-app>
</template>

<script setup>
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { ref, computed, onMounted, onBeforeUnmount } from 'vue';
import { usePortsStore } from '@/stores/ports';
import { useConfigStore } from '@/stores/config';
import { useCertsStore } from '@/stores/certs';
import { useAcmeStore } from '@/stores/acme';
import axios from 'axios';

const message = ref('');
let eventSource = null;

const { t } = useI18n({ useScope: 'global' })
const endpoint = import.meta.env.VITE_API_ENDPOINT;

const route = useRoute();
const router = useRouter();

const breadcrumbs = computed(() => {
  const { breadcrumb } = route.meta;
  if (breadcrumb) {
    return breadcrumb(route);
  } else {
    return [];
  }
});

const drawer = ref(undefined);

onMounted(async () => {
  const portsStore = usePortsStore();
  const configStore = useConfigStore();
  const certsStore = useCertsStore();
  const acmeStore = useAcmeStore();

  const token = localStorage.getItem('token');
  axios.defaults.headers.common['Authorization'] = `Bearer ${token}`;

  axios.interceptors.response.use((response) => {
    return response;
  }, (error) => {
    if (error.response.status === 401) {
      localStorage.removeItem('token')
      router.replace({ name: 'Login' })
    }
    return Promise.reject(error);
  });


  eventSource = new EventSource(`${endpoint}/events?token=${localStorage.getItem('token')}`);

  eventSource.onopen = (event) => {
    console.log('EventSource open:', event);
  };

  eventSource.onmessage = (event) => {
    const json = JSON.parse(event.data);
    switch (json.event) {
      case 'port_table_updated':
        portsStore.updateTable(json.entries);
        break;
      case 'port_status_updated':
        portsStore.updateStatus(json.id, json.status);
        break;
      case 'app_config_updated':
        configStore.update(json.config);
        break;
      case 'server_certs_updated':
        certsStore.update(json.items);
        break;
      case 'acme_updated':
        acmeStore.update(json.items);
        break;
    }
    message.value = event.data;
  };

  eventSource.onerror = (error) => {
    console.error('EventSource error:', error);
  };

  const { data: config } = await axios.get(`${endpoint}/config`);
  configStore.update(config);

  const { data: certs } = await axios.get(`${endpoint}/server_certs`);
  certsStore.update(certs);

  const { data: acme } = await axios.get(`${endpoint}/acme`);
  acmeStore.update(acme);

  const { data } = await axios.get(`${endpoint}/ports`);
  portsStore.updateTable(data);

  for (const port of data) {
    axios.get(`${endpoint}/ports/${port.id}/status`).then(({ data }) => {
      portsStore.updateStatus(port.id, data);
    });
  }
});

onBeforeUnmount(() => {
  if (eventSource) {
    eventSource.close();
  }
});

async function logout() {
  await axios.get(`${endpoint}/logout`);
  localStorage.removeItem('token');
  router.replace({ name: 'Login' });
}
</script>

