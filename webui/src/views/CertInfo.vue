<template>
    <v-tabs v-model="tab" align-tabs="center" show-arrows>
        <v-tab :value="1">{{ $t('ports.tab.status') }}</v-tab>
    </v-tabs>
    <v-window v-model="tab">
        <v-window-item :value="1">
            <v-table>
                <tbody>
                    <tr>
                        <td>{{ $t('certs.info.id') }}</td>
                        <td>{{ info.san.join(', ') }}</td>
                    </tr>
                    <tr>
                        <td>{{ $t('certs.info.id') }}</td>
                        <td>{{ info.fingerprint }}</td>
                    </tr>
                    <tr>
                        <td>{{ $t('certs.info.id') }}</td>
                        <td>{{ info.not_before }}</td>
                    </tr>
                    <tr>
                        <td>{{ $t('certs.info.id') }}</td>
                        <td>{{ info.not_after }}</td>
                    </tr>
                </tbody>
            </v-table>
            <v-divider></v-divider>
            <v-card-actions class="justify-end">
                <v-btn color="red" @click="deleteDialog = true">
                    {{ $t('certs.delete_cert') }}
                </v-btn>
            </v-card-actions>
            <v-dialog v-model="deleteDialog" width="auto">
                <v-card :title="$t('certs.delete_cert')">
                    <v-card-text>
                        {{ $t('certs.delete_cert_confirm', { name: route.params.id }) }}
                    </v-card-text>
                    <v-card-actions class="justify-end">
                        <v-btn @click="deleteDialog = false">Cancel</v-btn>
                        <v-btn color="red" @click="deleteCert">Delete</v-btn>
                    </v-card-actions>
                </v-card>
            </v-dialog>
        </v-window-item>
    </v-window>
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
        await axios.delete(`${endpoint}/certs/${route.params.id}`)
        router.replace({ name: 'Certificates' })
    } catch (err) {
        let { response: { data } } = err;
        error.value = data
    }
}
</script>
