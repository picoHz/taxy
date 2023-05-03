<template>
    <v-table>
        <tbody>
            <tr>
                <td>{{ $t('acme.info.provider') }}</td>
                <td>{{ info.provider }}</td>
            </tr>
            <tr>
                <td>{{ $t('acme.info.identifiers') }}</td>
                <td>{{ info.identifiers.join(', ') }}</td>
            </tr>
            <tr>
                <td>{{ $t('acme.info.challenge_type') }}</td>
                <td>{{ info.challenge_type }}</td>
            </tr>
        </tbody>
    </v-table>
    <v-divider></v-divider>
    <v-card-actions class="justify-end">
        <v-btn color="red" @click="deleteDialog = true">
            {{ $t('keyring.delete_acme.delete_acme') }}
        </v-btn>
    </v-card-actions>
    <v-dialog v-model="deleteDialog" width="auto">
        <v-card :title="$t('keyring.delete_acme.delete_acme')">
            <v-card-text>
                {{ $t('keyring.delete_acme.confirm', { id: route.params.id }) }}
            </v-card-text>
            <v-card-actions class="justify-end">
                <v-btn @click="deleteDialog = false">{{ $t('keyring.delete_acme.cancel') }}</v-btn>
                <v-btn color="red" @click="deleteCert">{{ $t('keyring.delete_acme.delete') }}</v-btn>
            </v-card-actions>
        </v-card>
    </v-dialog>
</template>
  
<script setup>
import { ref, computed } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useCertsStore } from '@/stores/certs';
import axios from 'axios';

const certsStore = useCertsStore();
const route = useRoute();
const router = useRouter();

const info = computed(() => certsStore.getStatusbyId(route.params.id));
const deleteDialog = ref(false);

const endpoint = import.meta.env.VITE_API_ENDPOINT;

async function deleteCert() {
    deleteDialog.value = false;
    try {
        await axios.delete(`${endpoint}/keyring/${route.params.id}`, {
            headers: {
                'Authorization': `Bearer ${localStorage.getItem('token')}`
            }
        })
        router.replace({ name: 'Certificate List' })
    } catch (err) {
        if (err.response.status === 401) {
            localStorage.removeItem('token')
            router.replace({ name: 'Login' })
        }
        let { response: { data } } = err;
        error.value = data
    }
}

</script>
