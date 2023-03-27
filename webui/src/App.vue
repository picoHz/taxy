<template>
  <router-view />
</template>

<script setup>
import { ref, onMounted, onBeforeUnmount } from 'vue';
import { usePortsStore } from '@/stores/ports';
import axious from 'axios';

const portsStore = usePortsStore();
const message = ref('');
let eventSource = null;

onMounted(async () => {
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
      case 'port_status_updated':
        portsStore.updateStatus(json.name, json.status);
    }
    message.value = event.data;
  };

  eventSource.onerror = (error) => {
    console.error('EventSource error:', error);
  };

  const { data } = await axious.get(`${endpoint}/ports`);
  portsStore.updateTable(data);

  for (const port of data) {
    axious.get(`${endpoint}/ports/${encodeURIComponent(port.name)}/status`).then(({ data }) => {
      portsStore.updateStatus(port.name, data);
    });
  }
});

onBeforeUnmount(() => {
  if (eventSource) {
    eventSource.close();
  }
});
</script>
