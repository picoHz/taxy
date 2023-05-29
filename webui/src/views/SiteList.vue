<template>
  <v-card max-width="800" class="mx-auto">
    <v-toolbar color="transparent" density="compact">
      <template v-slot:append>
        <v-btn prepend-icon="mdi-plus" @click="acmeDialog = true">
          {{ $t('acme.add_item') }}
        </v-btn>
      </template>
    </v-toolbar>
    <v-divider></v-divider>
    <v-list>
      <v-list-item v-if="acmeStore.list.length === 0" disabled>
        <v-list-item-title class="text-center">{{ $t('acme.no_items') }}</v-list-item-title>
      </v-list-item>
      <v-list-item prepend-icon="mdi-cloud-lock" v-for="item in acmeStore.list" :key="item.id" :title="item.provider"
        :subtitle="item.identifiers.join(', ')" :to="{ path: `/sites/${item.id}` }">
      </v-list-item>
    </v-list>
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

    <v-dialog :width="600" v-model="acmeDialog" width="auto">
      <v-form validate-on="submitAcmeForm" @submit.prevent="submitAcmeForm">
        <v-card>
          <v-card-title>
            {{ $t('acme.add_acme.title') }}
          </v-card-title>
          <v-card-text>
            <v-container>
              <v-row>
                <v-col cols="12" sm="12">
                  <v-select :label="$t('acme.provider')" :items="acmeProviders" v-model="acmeProvider" variant="outlined"
                    density="compact"></v-select>
                </v-col>
              </v-row>
            </v-container>
            <LetsEncrypt v-if="acmeProvider.startsWith('letsencrypt')" v-model="acmeModel"
              :staging="acmeProvider === 'letsencrypt-staging'"></LetsEncrypt>
          </v-card-text>

          <v-card-actions class="justify-end">
            <v-btn @click="acmeDialog = false">{{ $t('acme.add_acme.cancel') }}</v-btn>
            <v-btn :loading="loading" type="submit" color="primary">{{ $t('acme.add_acme.create') }}</v-btn>
          </v-card-actions>
        </v-card>
      </v-form>
    </v-dialog>

    <v-snackbar v-model="snackbar" :timeout="3000">
      {{ $t('acme.successfully_updated') }}
      <template v-slot:actions>
        <v-btn color="blue" variant="text" @click="snackbar = false">
          {{ $t('acme.snackbar_close') }}
        </v-btn>
      </template>
    </v-snackbar>
  </v-card>
</template>

<script setup>
import axios from 'axios';
import { ref } from 'vue'
import { useAcmeStore } from '@/stores/acme';
import LetsEncrypt from '@/acme/LetsEncrypt.vue';

const acmeStore = useAcmeStore();
const acmeDialog = ref(false);
const loading = ref(false);
const snackbar = ref(false);
const error = ref(null);

const acmeProvider = ref('letsencrypt');
const acmeModel = ref({});

const acmeProviders = [
  { title: "Let's Encrypt", value: 'letsencrypt' },
  { title: "Let's Encrypt (Staging)", value: 'letsencrypt-staging' },
];

const endpoint = import.meta.env.VITE_API_ENDPOINT;

async function submitAcmeForm(event) {
  let { valid } = await event;
  if (!valid) return;

  loading.value = true;
  try {
    await axios.post(`${endpoint}/acme`, acmeModel.value)
  } catch (err) {
    let { response: { data } } = err;
    error.value = data
  }
  loading.value = false;
  acmeDialog.value = false;
}
</script>
