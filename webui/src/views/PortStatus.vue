<template>
    <v-tabs v-model="tab" align-tabs="center" show-arrows>
        <v-tab :value="1">{{ $t('ports.tab.status') }}</v-tab>
        <v-tab :value="2">{{ $t('ports.tab.settings') }}</v-tab>
    </v-tabs>
    <v-window v-model="tab">
        <v-window-item :value="1">
            <v-table>
                <tbody>
                    <tr>
                        <td>{{ $t('ports.status.interface') }}</td>
                        <td>{{ config.listen }}</td>
                    </tr>
                    <tr>
                        <td>{{ $t('ports.status.state') }}</td>
                        <td>{{ $t(`ports.state.${state}`) }}</td>
                    </tr>
                    <tr>
                        <td>{{ $t('ports.status.uptime') }}</td>
                        <td>{{ uptime }}</td>
                    </tr>
                </tbody>
            </v-table>
            <LogViewer :logs="logs" />
        </v-window-item>
        <v-window-item :value="2">
            <port-config @submit="update" :entry="config" :loading="loading"></port-config>
            <v-divider></v-divider>
            <v-card-actions class="justify-end">
                <v-btn color="red" @click="deleteDialog = true">
                    {{ $t('ports.delete_port') }}
                </v-btn>
            </v-card-actions>
            <v-dialog v-model="deleteDialog" width="auto">
                <v-card :title="$t('ports.delete_port')">
                    <v-card-text>
                        {{ $t('ports.delete_port_confirm', { id: route.params.id }) }}
                    </v-card-text>
                    <v-card-actions class="justify-end">
                        <v-btn @click="deleteDialog = false">Cancel</v-btn>
                        <v-btn color="red" @click="deletePort">Delete</v-btn>
                    </v-card-actions>
                </v-card>
            </v-dialog>
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
            <v-snackbar v-model="snackbar" :timeout="3000">
                {{ $t('ports.successfully_updated') }}
                <template v-slot:actions>
                    <v-btn color="blue" variant="text" @click="snackbar = false">
                        {{ $t('ports.snackbar_close') }}
                    </v-btn>
                </template>
            </v-snackbar>
        </v-window-item>
    </v-window>
</template>
  
<script setup>
import LogViewer from '@/components/LogViewer.vue';
import { ref, computed, onMounted } from 'vue';
import PortConfig from '@/components/PortConfig.vue';
import { useRoute, useRouter } from 'vue-router';
import { usePortsStore } from '@/stores/ports';
import formatDuration from 'format-duration';
import axios from 'axios';

const portsStore = usePortsStore();
const router = useRouter();
const route = useRoute();
const tab = ref(1);
const loading = ref(false);
const snackbar = ref(false);
const error = ref(null);
const deleteDialog = ref(false);
const logs = ref([]);

const now = ref(Date.now())

setInterval(() => {
    now.value = Date.now()
}, 1000)

const state = computed(() => portsStore.getStateByName(route.params.id));
const status = computed(() => portsStore.getStatusByName(route.params.id));
const config = computed(() => portsStore.table.find(({ id }) => route.params.id === id) || {});
const startedAt = computed(() => status.value.started_at);
const uptime = computed(() => startedAt.value ? formatDuration(now.value - startedAt.value * 1000) : 'n/a');

const endpoint = import.meta.env.VITE_API_ENDPOINT;

async function update(data) {
    loading.value = true;
    try {
        await axios.put(`${endpoint}/ports/${route.params.id}`, data)
        snackbar.value = true
    } catch (err) {
        let { response: { data } } = err;
        error.value = data
    }
    loading.value = false;
}

async function deletePort() {
    deleteDialog.value = false;
    try {
        await axios.delete(`${endpoint}/ports/${route.params.id}`)
        router.replace({ name: 'List' })
    } catch (err) {
        let { response: { data } } = err;
        error.value = data
    }
}

onMounted(async () => {
    const { data } = await axios.get(`${endpoint}/ports/${route.params.id}/log`);
    logs.value = data
})
</script>
