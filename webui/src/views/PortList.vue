<template>
  <v-card max-width="800" class="mx-auto">
    <v-toolbar color="transparent" density="compact">
      <template v-slot:append>
        <v-btn :to="{ path: '/ports/new' }" prepend-icon="mdi-plus">
          {{ $t('ports.new_port') }}
        </v-btn>
      </template>
    </v-toolbar>
    <v-divider></v-divider>
    <v-list>
      <v-list-item v-if="portsStore.table.length === 0" disabled>
        <v-list-item-title class="text-center">{{ $t('ports.no_ports') }}</v-list-item-title>
      </v-list-item>
      <v-list-item v-for="item in portsStore.table" :key="item.listen" :title="item.name" :subtitle="item.listen"
        :value="item.listen" :to="{ path: `/ports/${encodeURIComponent(item.name)}` }">

        <template v-slot:prepend>
          <v-icon v-if="getStateByName(item.name) === 'listening'" icon="$success" color="green"></v-icon>
          <v-icon v-else icon="$error" color="error"></v-icon>
        </template>
      </v-list-item>
    </v-list>
  </v-card>
</template>

<script setup>
import { storeToRefs } from 'pinia'
import { usePortsStore } from '@/stores/ports';

const portsStore = usePortsStore();
const { getStateByName } = storeToRefs(portsStore);
</script>
