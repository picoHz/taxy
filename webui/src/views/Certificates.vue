<template>
  <v-card max-width="800" class="mx-auto">
    <v-toolbar color="transparent" density="compact">
      <template v-slot:append>
        <v-btn prepend-icon="mdi-plus">
          {{ $t('keyring.add_item') }}
          <v-menu activator="parent">
            <v-list>
              <v-list-item prepend-icon="mdi-upload" @click="uploadDialog = true">
                <v-list-item-title>{{ $t('keyring.upload.upload') }}</v-list-item-title>
              </v-list-item>
              <v-list-item prepend-icon="mdi-file-sign" @click="selfSignedDialog = true">
                <v-list-item-title>{{ $t('keyring.self_sign.self_sign') }}</v-list-item-title>
              </v-list-item>
              <v-list-item prepend-icon="mdi-cloud-lock" @click="acmeDialog = true">
                <v-list-item-title>{{ $t('keyring.acme.acme') }}</v-list-item-title>
              </v-list-item>
            </v-list>
          </v-menu>
        </v-btn>
      </template>
    </v-toolbar>
    <v-divider></v-divider>
    <v-list>
      <v-list-item v-if="certsStore.list.length === 0" disabled>
        <v-list-item-title class="text-center">{{ $t('keyring.no_certs') }}</v-list-item-title>
      </v-list-item>
      <v-list-item prepend-icon="mdi-cloud-lock" v-for="item in certsStore.list.filter(item => item.type === 'acme')"
        :key="item.id" :title="item.provider" :subtitle="item.identifiers.join(', ')"
        :to="{ path: `/keyring/acme/${item.id}` }">
      </v-list-item>
      <v-list-item prepend-icon="mdi-file-certificate"
        v-for="item in certsStore.list.filter(item => item.type === 'server_cert')" :key="item.id"
        :title="item.san.join(', ')" :subtitle="item.id" :value="item.listen" :to="{ path: `/keyring/certs/${item.id}` }">
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

    <v-dialog :width="600" v-model="uploadDialog" width="auto">
      <v-form validate-on="submitUploadForm" @submit.prevent="submitUploadForm">
        <v-card>
          <v-card-title>
            {{ $t('keyring.upload.title') }}
          </v-card-title>
          <v-card-text>
            <v-container>
              <v-row>
                <v-col cols="12" sm="12">
                  <v-file-input v-model="chainFile" :rules="chainFileRules" :label="$t('keyring.upload.chain')"
                    variant="outlined" density="compact" prepend-icon="mdi-certificate" :hint="$t('keyring.upload.hint')"
                    persistent-hint></v-file-input>
                </v-col>
                <v-col cols="12" sm="12">
                  <v-file-input v-model="keyFile" :rules="keyFileRules" required :label="$t('keyring.upload.key')"
                    variant="outlined" density="compact" prepend-icon="mdi-key" :hint="$t('keyring.upload.hint')"
                    persistent-hint></v-file-input>
                </v-col>
              </v-row>
            </v-container>
          </v-card-text>

          <v-card-actions class="justify-end">
            <v-btn @click="uploadDialog = false">{{ $t('keyring.upload.cancel') }}</v-btn>
            <v-btn :loading="loading" type="submit" color="primary">{{ $t('keyring.upload.upload') }}</v-btn>
          </v-card-actions>
        </v-card>
      </v-form>
    </v-dialog>

    <v-dialog :width="600" v-model="selfSignedDialog" width="auto">
      <v-form validate-on="submitSelfSignedForm" @submit.prevent="submitSelfSignedForm">
        <v-card>
          <v-card-title>
            {{ $t('keyring.self_sign.title') }}
          </v-card-title>
          <v-card-text>
            <v-container>
              <v-row>
                <v-col cols="12" sm="12">
                  <v-text-field :label="$t('keyring.self_sign.subject_alternative_names')" variant="outlined"
                    v-model="selfSignedRequest.san" autocapitalize="off" :hint="$t('keyring.self_sign.hint')"
                    :rules="tlsServerNamesRules" density="compact" persistent-hint></v-text-field>
                </v-col>
              </v-row>
            </v-container>
          </v-card-text>

          <v-card-actions class="justify-end">
            <v-btn @click="selfSignedDialog = false">{{ $t('keyring.self_sign.cancel') }}</v-btn>
            <v-btn :loading="loading" type="submit" color="primary">{{ $t('keyring.self_sign.create') }}</v-btn>
          </v-card-actions>
        </v-card>
      </v-form>
    </v-dialog>

    <v-dialog :width="600" v-model="acmeDialog" width="auto">
      <v-form validate-on="submitAcmeForm" @submit.prevent="submitAcmeForm">
        <v-card>
          <v-card-title>
            {{ $t('keyring.acme.title') }}
          </v-card-title>
          <v-card-text>
            <v-container>
              <v-row>
                <v-col cols="12" sm="12">
                  <v-select :label="$t('keyring.acme.provider')" :items="acmeProviders" v-model="acmeProvider"
                    variant="outlined" density="compact"></v-select>
                </v-col>
              </v-row>
            </v-container>
            <LetsEncrypt v-model="acmeModel" :staging="acmeProvider === 'letsencrypt-staging'"></LetsEncrypt>
          </v-card-text>

          <v-card-actions class="justify-end">
            <v-btn @click="acmeDialog = false">{{ $t('keyring.acme.cancel') }}</v-btn>
            <v-btn :loading="loading" type="submit" color="primary">{{ $t('keyring.acme.create') }}</v-btn>
          </v-card-actions>
        </v-card>
      </v-form>
    </v-dialog>

    <v-snackbar v-model="snackbar" :timeout="3000">
      {{ $t('keyring.successfully_updated') }}
      <template v-slot:actions>
        <v-btn color="blue" variant="text" @click="snackbar = false">
          {{ $t('keyring.snackbar_close') }}
        </v-btn>
      </template>
    </v-snackbar>
  </v-card>
</template>

<script setup>
import axios from 'axios';
import { ref, reactive } from 'vue'
import { useCertsStore } from '@/stores/certs';
import { useI18n } from 'vue-i18n'
import { parseTlsServerNames } from '@/utils/validators'
import LetsEncrypt from '@/acme/LetsEncrypt.vue';

const { t } = useI18n({ useScope: 'global' })

const certsStore = useCertsStore();
const uploadDialog = ref(false);
const chainFile = ref([]);
const keyFile = ref([]);
const selfSignedDialog = ref(false);
const acmeDialog = ref(false);
const selfSignedRequest = reactive({
  san: ""
});
const loading = ref(false);
const snackbar = ref(false);
const error = ref(null);

const acmeProvider = ref('letsencrypt');
const acmeModel = ref({});

const acmeProviders = [
  { title: "Let's Encrypt", value: 'letsencrypt' },
  { title: "Let's Encrypt (Staging)", value: 'letsencrypt-staging' }
];

const endpoint = import.meta.env.VITE_API_ENDPOINT;

async function submitUploadForm(event) {
  let { valid } = await event;
  if (!valid) return;

  loading.value = true;

  const formData = new FormData();
  formData.append('chain', chainFile.value[0]);
  formData.append('key', keyFile.value[0]);
  console.log(formData)

  try {
    await axios.post(`${endpoint}/keyring/upload`, formData, {
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
    await axios.post(`${endpoint}/keyring/self_signed`, {
      san: parseTlsServerNames(selfSignedRequest.san)
    })
  } catch (err) {
    let { response: { data } } = err;
    error.value = data
  }
  loading.value = false;
  selfSignedDialog.value = false;
}

async function submitAcmeForm(event) {
  let { valid } = await event;
  if (!valid) return;

  loading.value = true;
  try {
    await axios.post(`${endpoint}/keyring/acme`, acmeModel.value)
  } catch (err) {
    let { response: { data } } = err;
    error.value = data
  }
  loading.value = false;
  acmeDialog.value = false;
}

const chainFileRules = [
  value => {
    if (value.length > 0) return true
    return t('keyring.upload.rule_chain')
  },
]

const keyFileRules = [
  value => {
    if (value.length > 0) return true
    return t('keyring.upload.rule_key')
  },
]

const tlsServerNamesRules = [
  value => {
    const list = parseTlsServerNames(value)
    if (list.length > 0) return true
    return t('keyring.self_sign.rule')
  },
]

</script>
