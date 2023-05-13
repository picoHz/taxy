<template>
    <v-table>
        <tbody>
            <tr>
                <td>{{ $t('server_certs.info.san') }}</td>
                <td>{{ info.san.join(', ') }}</td>
            </tr>
            <tr>
                <td>{{ $t('server_certs.info.fingerprint') }}</td>
                <td>{{ info.fingerprint }}</td>
            </tr>
            <tr>
                <td>{{ $t('server_certs.info.issuer') }}</td>
                <td>{{ info.issuer }}</td>
            </tr>
            <tr v-if="info.root_cert">
                <td>{{ $t('server_certs.info.root_cert') }}</td>
                <td>{{ info.root_cert }}</td>
            </tr>
            <tr>
                <td>{{ $t('server_certs.info.not_before') }}</td>
                <td>{{ (new Date(info.not_before * 1000)).toISOString() }}</td>
            </tr>
            <tr>
                <td>{{ $t('server_certs.info.not_after') }}</td>
                <td>{{ (new Date(info.not_after * 1000)).toISOString() }}</td>
            </tr>
        </tbody>
    </v-table>
    <v-divider></v-divider>
    <v-card-actions class="justify-end">
        <v-btn color="red" @click="deleteDialog = true">
            {{ $t('server_certs.delete_cert.delete_cert') }}
        </v-btn>
    </v-card-actions>
    <v-dialog v-model="deleteDialog" width="auto">
        <v-card :title="$t('server_certs.delete_cert.delete_cert')">
            <v-card-text>
                {{ $t('server_certs.delete_cert.confirm', { id: route.params.id }) }}
            </v-card-text>
            <v-card-actions class="justify-end">
                <v-btn @click="deleteDialog = false">{{ $t('server_certs.delete_cert.cancel') }}</v-btn>
                <v-btn color="red" @click="deleteCert">{{ $t('server_certs.delete_cert.delete') }}</v-btn>
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

const tab = ref(1);
const deleteDialog = ref(false);

const endpoint = import.meta.env.VITE_API_ENDPOINT;

async function deleteCert() {
    deleteDialog.value = false;
    try {
        await axios.delete(`${endpoint}/server_certs/${route.params.id}`)
        router.replace({ name: 'Server Certificate List' })
    } catch (err) {
        let { response: { data } } = err;
        error.value = data
    }
}
</script>
