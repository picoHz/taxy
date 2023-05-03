<template>
    <port-config @submit="register"></port-config>
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
</template>
  
<script setup>
import { ref } from 'vue';
import PortConfig from '@/components/PortConfig.vue'
import { useRouter } from 'vue-router';
import axios from 'axios';

const router = useRouter();
const endpoint = import.meta.env.VITE_API_ENDPOINT;

const loading = ref(false);
const error = ref(null);

async function register(data) {
    loading.value = true;
    try {
        await axios.post(`${endpoint}/ports`, data, {
            headers: {
                'Authorization': `Bearer ${localStorage.getItem('token')}`
            }
        })
        router.push({ name: 'List' })
    } catch (err) {
        if (err.response.status === 401) {
            localStorage.removeItem('token')
            router.replace({ name: 'Login' })
        }
        let { response: { data } } = err;
        error.value = data
    }
    loading.value = false;
}
</script>
