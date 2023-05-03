<template>
  <v-app>
    <v-layout>
      <v-main>
        <v-toolbar color="primary" dark extended flat>
        </v-toolbar>

        <v-card class="mx-auto" max-width="300" style="margin-top: -64px;">
          <v-card-text>
            <v-form validate-on="submit" @submit.prevent="submit">
              <v-text-field variant="filled" v-model="userName" :rules="rules" label="User"></v-text-field>
              <v-text-field variant="filled" type="password" v-model="password" :rules="rules"
                label="Password"></v-text-field>
              <v-btn type="submit" block class="mt-2">Login</v-btn>
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
const userName = ref("");
const password = ref("");

const endpoint = import.meta.env.VITE_API_ENDPOINT;

async function submit(event) {
  let { valid } = await event;
  if (valid) {
    try {
      const { data } = await axios.post(`${endpoint}/login`, { user: userName.value, password: password.value })
      localStorage.setItem('token', data.token)
      router.push({ name: 'List' })
    } catch (err) {
      let { response: { data } } = err;
      error.value = data
    }
  }
}

const rules = [
  (v) => !!v || t('login.required'),
]

</script>
