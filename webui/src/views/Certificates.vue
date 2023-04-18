<template>
  <v-card max-width="800" class="mx-auto">
    <v-toolbar color="transparent" density="compact">
      <v-toolbar-title>{{ $t('certs.certs_paths.certs_paths') }}</v-toolbar-title>
    </v-toolbar>
    <v-divider></v-divider>
    <v-container>
      <v-form>
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
  </v-card>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { useConfigStore } from '@/stores/config';

const configStore = useConfigStore();
const certPaths = ref("");
const loading = ref(false);

onMounted(() => {
  certPaths.value = configStore.app.certs.search_paths.join("\n");
})
</script>
