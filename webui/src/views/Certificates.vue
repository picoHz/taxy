<template>
  <v-card max-width="800" class="mx-auto">
    <v-toolbar color="transparent" density="compact">
      <template v-slot:append>
        <v-btn prepend-icon="mdi-plus">
          {{ $t('certs.add_cert') }}
          <v-menu activator="parent">
            <v-list>
              <v-list-item @click="uploadDialog = true">
                <v-list-item-title>{{ $t('certs.upload') }}</v-list-item-title>
              </v-list-item>
              <v-list-item @click="selfSignedDialog = true">
                <v-list-item-title>{{ $t('certs.self_signed') }}</v-list-item-title>
              </v-list-item>
            </v-list>
          </v-menu>
        </v-btn>
      </template>
    </v-toolbar>
    <v-divider></v-divider>
    <v-list>
      <v-list-item v-if="certsStore.list.length === 0" disabled>
        <v-list-item-title class="text-center">{{ $t('certs.no_certs') }}</v-list-item-title>
      </v-list-item>
      <v-list-item v-for="item in certsStore.list" :key="item.id" :title="item.san.join(', ')" :subtitle="item.id"
        :value="item.listen" :to="{ path: `/certs/${item.id}` }">
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

    <v-dialog :width="600" v-model="uploadDialog" width="auto">
      <v-form validate-on="submitUploadForm" @submit.prevent="submitUploadForm">
        <v-card>
          <v-card-title>
            Upload Certificate
          </v-card-title>
          <v-card-text>
            <v-container>
              <v-row>
                <v-col cols="12" sm="12">
                  <v-file-input v-model="chainFile" :rules="chainFileRules" label="Certificate Chain" variant="outlined"
                    density="compact" prepend-icon="mdi-certificate" hint="Only PEM file format is supported."
                    persistent-hint></v-file-input>
                </v-col>
                <v-col cols="12" sm="12">
                  <v-file-input v-model="keyFile" :rules="keyFileRules" required label="Private Key" variant="outlined"
                    density="compact" prepend-icon="mdi-key" hint="Only PEM file format is supported."
                    persistent-hint></v-file-input>
                </v-col>
              </v-row>
            </v-container>
          </v-card-text>

          <v-card-actions class="justify-end">
            <v-btn @click="uploadDialog = false">Cancel</v-btn>
            <v-btn :loading="loading" type="submit" color="primary">Upload</v-btn>
          </v-card-actions>
        </v-card>
      </v-form>
    </v-dialog>

    <v-dialog :width="600" v-model="selfSignedDialog" width="auto">
      <v-form validate-on="submitSelfSignedForm" @submit.prevent="submitSelfSignedForm">
        <v-card>
          <v-card-title>
            New Self-signed Certificate
          </v-card-title>
          <v-card-text>
            <v-container>
              <v-row>
                <v-col cols="12" sm="12">
                  <v-text-field :label="$t('ports.config.tls_term.server_names.server_names')" variant="outlined"
                    v-model="selfSignedRequest.san" :hint="$t('ports.config.tls_term.server_names.hint')"
                    :rules="tlsServerNamesRules" density="compact" persistent-hint></v-text-field>
                </v-col>
              </v-row>
            </v-container>
          </v-card-text>

          <v-card-actions class="justify-end">
            <v-btn @click="selfSignedDialog = false">Cancel</v-btn>
            <v-btn :loading="loading" type="submit" color="primary">Create</v-btn>
          </v-card-actions>
        </v-card>
      </v-form>
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
import { ref, reactive, onMounted } from 'vue'
import { useConfigStore } from '@/stores/config';
import { useCertsStore } from '@/stores/certs';
import { useI18n } from 'vue-i18n'
import { parseTlsServerNames } from '@/utils/validators'

const { t } = useI18n({ useScope: 'global' })

const configStore = useConfigStore();
const certsStore = useCertsStore();
const certPaths = ref("");
const uploadDialog = ref(false);
const chainFile = ref([]);
const keyFile = ref([]);
const selfSignedDialog = ref(false);
const selfSignedRequest = reactive({
  san: ""
});
const loading = ref(false);
const snackbar = ref(false);
const error = ref(null);

const endpoint = import.meta.env.VITE_API_ENDPOINT;

onMounted(() => {
  const configStore = useConfigStore();
  certPaths.value = configStore.app.certs.search_paths.join("\n");
})

async function submitUploadForm(event) {
  let { valid } = await event;
  if (!valid) return;

  loading.value = true;

  const formData = new FormData();
  formData.append('chain', chainFile.value[0]);
  formData.append('key', keyFile.value[0]);
  console.log(formData)

  try {
    await axios.post(`${endpoint}/certs/upload`, formData, {
      headers: {
        'Content-Type': 'multipart/form-data',
      },
    })
  } catch (err) {
    let { response: { data } } = err;
    error.value = data
  }
  loading.value = false;
  uploadDialog.value = false;
}

async function submitSelfSignedForm(event) {
  let { valid } = await event;
  if (!valid) return;

  loading.value = true;
  try {
    await axios.post(`${endpoint}/certs/self_signed`, {
      san: parseTlsServerNames(selfSignedRequest.san)
    })
  } catch (err) {
    let { response: { data } } = err;
    error.value = data
  }
  loading.value = false;
  selfSignedDialog.value = false;
}

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

const chainFileRules = [
  value => {
    if (value.length > 0) return true
    return t('ports.config.tls_term.server_names.rule')
  },
]

const keyFileRules = [
  value => {
    if (value.length > 0) return true
    return t('ports.config.tls_term.server_names.rule')
  },
]

const tlsServerNamesRules = [
  value => {
    const list = parseTlsServerNames(value)
    if (list.length > 0) return true
    return t('ports.config.tls_term.server_names.rule')
  },
]

</script>
