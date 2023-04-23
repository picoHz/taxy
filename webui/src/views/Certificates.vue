<template>
  <v-card max-width="800" class="mx-auto">
    <v-list>
      <v-list-item v-if="certsStore.list.length === 0" disabled>
        <v-list-item-title class="text-center">{{ $t('certs.no_certs') }}</v-list-item-title>
      </v-list-item>
      <v-list-item v-for="item in certsStore.list" :key="item.id" :title="item.san.join(', ')" :subtitle="item.id"
        :value="item.listen" :to="{ path: `/ports/${encodeURIComponent(item.name)}` }">
      </v-list-item>
    </v-list>
    <v-toolbar color="transparent" density="compact">
      <v-toolbar-title>{{ $t('certs.certs_paths.certs_paths') }}</v-toolbar-title>
    </v-toolbar>
    <v-divider></v-divider>
    <v-container>
      <v-form validate-on="submitForm" @submit.prevent="submitForm">
        <p class="mb-4">{{ $t('certs.certs_paths.description') }}</p>
        <v-textarea v-model="certPaths" :label="$t('certs.certs_paths.certs_paths')" variant="outlined" density="compact"
          :placeholder="$t('certs.certs_paths.placeholder')">
        </v-textarea>
        <div class="d-flex justify-end">
          <v-btn :loading="loading" type="submit" color="primary">
            {{ $t('certs.certs_paths.update') }}
          </v-btn>
        </div>
      </v-form>
    </v-container>
    <v-dialog v-model="error" width="auto">
      <v-card title="Error">
        <v-card-text v-if="error.error">
          {{ $t(`error.${error.error.message}`, error.error) }}
        </v-card-text>
        <v-card-text v-else>
          {{ error.message }}
        </v-card-text>
        <v-card-actions>
          <v-btn color="primary" block @click="error = false">Close</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
    <v-snackbar v-model="snackbar" :timeout="3000">
      {{ $t('certs.successfully_updated') }}
      <template v-slot:actions>
        <v-btn color="blue" variant="text" @click="snackbar = false">
          {{ $t('certs.snackbar_close') }}
        </v-btn>
      </template>
    </v-snackbar>
  </v-card>
</template>

<script setup>
import axios from 'axios';
import { ref, onMounted } from 'vue'
import { useConfigStore } from '@/stores/config';
import { useCertsStore } from '@/stores/certs';

const configStore = useConfigStore();
const certsStore = useCertsStore();
const certPaths = ref("");
const loading = ref(false);
const snackbar = ref(false);
const error = ref(null);

const endpoint = import.meta.env.VITE_API_ENDPOINT;

onMounted(() => {
  const configStore = useConfigStore();
  certPaths.value = configStore.app.certs.search_paths.join("\n");
})

async function submitForm(event) {
  let { valid } = await event;
  if (valid) {
    const paths = certPaths.value.split("\n")
      .map((path) => path.trim())
      .filter((path) => path.length);
    configStore.setCertSearchPaths(paths);
    loading.value = true;
    try {
      await axios.put(`${endpoint}/config`, configStore.app)
      snackbar.value = true
    } catch (err) {
      let { response: { data } } = err;
      error.value = data
    }
    loading.value = false;
  }
}
</script>
