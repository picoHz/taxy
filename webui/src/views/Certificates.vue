<template>
  <v-card max-width="800" class="mx-auto">
    <v-toolbar color="transparent" density="compact">
      <template v-slot:append>
        <v-btn prepend-icon="mdi-plus">
          {{ $t('certs.add_item') }}
          <v-menu activator="parent">
            <v-list>
              <v-list-item prepend-icon="mdi-upload" @click="uploadDialog = true">
                <v-list-item-title>{{ $t('certs.upload.upload') }}</v-list-item-title>
              </v-list-item>
              <v-list-item prepend-icon="mdi-file-sign" @click="selfSignedDialog = true">
                <v-list-item-title>{{ $t('certs.self_sign.self_sign') }}</v-list-item-title>
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
      <v-list-item prepend-icon="mdi-file-certificate" v-for="item in certsStore.list" :key="item.id"
        :title="item.san.join(', ')" :subtitle="item.id" :value="item.listen" :to="{ path: `/certs/${item.id}` }">
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
            {{ $t('certs.upload.title') }}
          </v-card-title>
          <v-card-text>
            <v-container>
              <v-row>
                <v-col cols="12" sm="12">
                  <v-file-input v-model="chainFile" :rules="chainFileRules" :label="$t('certs.upload.chain')"
                    variant="outlined" density="compact" prepend-icon="mdi-certificate" :hint="$t('certs.upload.hint')"
                    persistent-hint></v-file-input>
                </v-col>
                <v-col cols="12" sm="12">
                  <v-file-input v-model="keyFile" :rules="keyFileRules" required :label="$t('certs.upload.key')"
                    variant="outlined" density="compact" prepend-icon="mdi-key" :hint="$t('certs.upload.hint')"
                    persistent-hint></v-file-input>
                </v-col>
              </v-row>
            </v-container>
          </v-card-text>

          <v-card-actions class="justify-end">
            <v-btn @click="uploadDialog = false">{{ $t('certs.upload.cancel') }}</v-btn>
            <v-btn :loading="loading" type="submit" color="primary">{{ $t('certs.upload.upload') }}</v-btn>
          </v-card-actions>
        </v-card>
      </v-form>
    </v-dialog>

    <v-dialog :width="600" v-model="selfSignedDialog" width="auto">
      <v-form validate-on="submitSelfSignedForm" @submit.prevent="submitSelfSignedForm">
        <v-card>
          <v-card-title>
            {{ $t('certs.self_sign.title') }}
          </v-card-title>
          <v-card-text>
            <v-container>
              <v-row>
                <v-col cols="12" sm="12">
                  <v-text-field :label="$t('certs.self_sign.subject_alternative_names')" variant="outlined"
                    v-model="selfSignedRequest.san" autocapitalize="off" :hint="$t('certs.self_sign.hint')"
                    :rules="tlsServerNamesRules" density="compact" persistent-hint></v-text-field>
                </v-col>
              </v-row>
            </v-container>
          </v-card-text>

          <v-card-actions class="justify-end">
            <v-btn @click="selfSignedDialog = false">{{ $t('certs.self_sign.cancel') }}</v-btn>
            <v-btn :loading="loading" type="submit" color="primary">{{ $t('certs.self_sign.create') }}</v-btn>
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
import { ref, reactive } from 'vue'
import { useCertsStore } from '@/stores/certs';
import { useI18n } from 'vue-i18n'
import { parseTlsServerNames } from '@/utils/validators'

const { t } = useI18n({ useScope: 'global' })

const certsStore = useCertsStore();
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

async function submitUploadForm(event) {
  let { valid } = await event;
  if (!valid) return;

  loading.value = true;

  const formData = new FormData();
  formData.append('chain', chainFile.value[0]);
  formData.append('key', keyFile.value[0]);
  console.log(formData)

  try {
    await axios.post(`${endpoint}/server_certs/upload`, formData, {
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
    await axios.post(`${endpoint}/server_certs/self_sign`, {
      san: parseTlsServerNames(selfSignedRequest.san)
    })
  } catch (err) {
    let { response: { data } } = err;
    error.value = data
  }
  loading.value = false;
  selfSignedDialog.value = false;
}

const chainFileRules = [
  value => {
    if (value.length > 0) return true
    return t('certs.upload.rule_chain')
  },
]

const keyFileRules = [
  value => {
    if (value.length > 0) return true
    return t('certs.upload.rule_key')
  },
]

const tlsServerNamesRules = [
  value => {
    const list = parseTlsServerNames(value)
    if (list.length > 0) return true
    return t('certs.self_sign.rule')
  },
]

</script>
