<template>
  <v-app>
    <v-layout>
      <v-main>
        <v-toolbar color="primary" dark extended flat>
        </v-toolbar>

        <v-card class="mx-auto" max-width="300" style="margin-top: -64px;">
          <v-alert v-if="error" color="error" icon="$error" :text="$t('login.login_failed')"></v-alert>
          <v-card-text>
            <v-form validate-on="submit" @submit.prevent="submit">
              <v-text-field variant="filled" v-model="username" :rules="nameRules"
                :label="$t('login.username')"></v-text-field>
              <v-text-field variant="filled" type="password" v-model="password" :rules="passwordRules"
                :label="$t('login.password')"></v-text-field>
              <v-btn type="submit" :loading="loading" block class="mt-2">{{ t('login.login') }}</v-btn>
            </v-form>
          </v-card-text>
        </v-card>
      </v-main>
    </v-layout>
  </v-app>
</template>

<script setup>
import { ref } from 'vue';
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n'
import axios from 'axios';

const { t } = useI18n({ useScope: 'global' })

const router = useRouter();
const username = ref("");
const password = ref("");
const loading = ref(false);
const error = ref(null);

const endpoint = import.meta.env.VITE_API_ENDPOINT;

async function submit(event) {
  let { valid } = await event;
  if (valid) {
    loading.value = true;
    try {
      const { data } = await axios.post(`${endpoint}/login`, { username: username.value, password: password.value })
      localStorage.setItem('token', data.token)
      router.push({ name: 'List' })
    } catch (err) {
      let { response: { data } } = err;
      error.value = data
    }
    loading.value = false;
  }
}

const nameRules = [
  (v) => !!v || t('login.username_required'),
]

const passwordRules = [
  (v) => !!v || t('login.password_required'),
]

</script>
