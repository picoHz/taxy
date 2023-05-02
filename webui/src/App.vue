<template>
  <router-view />
</template>

<script setup>
import { ref, onMounted, onBeforeUnmount } from 'vue';
import { usePortsStore } from '@/stores/ports';
import { useConfigStore } from '@/stores/config';
import { useCertsStore } from '@/stores/certs';
import axious from 'axios';

const message = ref('');
let eventSource = null;

onMounted(async () => {
  const portsStore = usePortsStore();
  const configStore = useConfigStore();
  const certsStore = useCertsStore();

  const endpoint = import.meta.env.VITE_API_ENDPOINT;
  eventSource = new EventSource(`${endpoint}/events`);

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
      case 'keyring_updated':
        certsStore.update(json.items);
        break;
    }
    message.value = event.data;
  };

  eventSource.onerror = (error) => {
    console.error('EventSource error:', error);
  };

  const { data: config } = await axious.get(`${endpoint}/config`);
  configStore.update(config);

  const { data: certs } = await axious.get(`${endpoint}/keyring`);
  certsStore.update(certs);

  const { data } = await axious.get(`${endpoint}/ports`);
  portsStore.updateTable(data);

  for (const port of data) {
    axious.get(`${endpoint}/ports/${port.id}/status`).then(({ data }) => {
      portsStore.updateStatus(port.id, data);
    });
  }
});

onBeforeUnmount(() => {
  if (eventSource) {
    eventSource.close();
  }
});
</script>
