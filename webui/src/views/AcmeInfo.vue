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
    <LogViewer :logs="logs" />
    <v-divider></v-divider>
    <v-card-actions class="justify-end">
        <v-btn color="red" @click="deleteDialog = true">
            {{ $t('acme.delete_acme.delete_acme') }}
        </v-btn>
    </v-card-actions>
    <v-dialog v-model="deleteDialog" width="auto">
        <v-card :title="$t('acme.delete_acme.delete_acme')">
            <v-card-text>
                {{ $t('acme.delete_acme.confirm', { id: route.params.id }) }}
            </v-card-text>
            <v-card-actions class="justify-end">
                <v-btn @click="deleteDialog = false">{{ $t('acme.delete_acme.cancel') }}</v-btn>
                <v-btn color="red" @click="deleteCert">{{ $t('acme.delete_acme.delete') }}</v-btn>
            </v-card-actions>
        </v-card>
    </v-dialog>
</template>
  
<script setup>
import LogViewer from '@/components/LogViewer.vue';
import { ref, computed, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useAcmeStore } from '@/stores/acme';
import axios from 'axios';

const acmeStore = useAcmeStore();
const route = useRoute();
const router = useRouter();

const info = computed(() => acmeStore.getStatusbyId(route.params.id));
const deleteDialog = ref(false);
const logs = ref([]);

const endpoint = import.meta.env.VITE_API_ENDPOINT;

async function deleteCert() {
    deleteDialog.value = false;
    try {
        await axios.delete(`${endpoint}/acme/${route.params.id}`)
        router.replace({ name: 'ACME List' })
    } catch (err) {
        let { response: { data } } = err;
        error.value = data
    }
}

onMounted(async () => {
    let since = null;
    for (; ;) {
        try {
            const { data } = await axios.get(`${endpoint}/acme/${route.params.id}/log`, {
                params: { since }
            });
            logs.value = logs.value.concat(data)
            const last = logs.value[logs.value.length - 1]
            since = last ? last.timestamp : Date.now() / 1000;
        } catch (err) {
            if (err.response.status !== 408) {
                throw err;
            }
        }
    }
})
</script>
