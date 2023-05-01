<template>
    <v-form validate-on="submitForm" @submit.prevent="submitForm">
        <v-toolbar color="transparent" density="compact">
            <v-toolbar-title>
                {{ $t('ports.config.listener') }}
            </v-toolbar-title>
        </v-toolbar>
        <v-divider></v-divider>
        <v-container>
            <v-row>
                <v-col cols="12" sm="2">
                    <v-select :label="$t('ports.config.protocol')" :items="protocols" v-model="formData.protocol"
                        variant="outlined" density="compact"></v-select>
                </v-col>

                <v-col cols="24" sm="6">
                    <v-text-field :label="$t('ports.config.interface')" v-model="formData.ifs" required variant="outlined"
                        density="compact" :rules="interfaceRules"></v-text-field>
                </v-col>

                <v-col cols="24" sm="4">
                    <v-text-field :label="$t('ports.config.port')" v-model="formData.port" required min="1" max="65535"
                        variant="outlined" density="compact" type="number" :rules="portRules"></v-text-field>
                </v-col>
            </v-row>
        </v-container>
        <div v-if="formData.protocol === 'tls'">
            <v-toolbar color="transparent" density="compact">
                <v-toolbar-title>
                    {{ $t('ports.config.tls_term.tls_term') }}
                </v-toolbar-title>
            </v-toolbar>
            <v-divider></v-divider>
            <v-container>
                <v-row>
                    <v-col cols="12" sm="12">
                        <v-text-field :label="$t('ports.config.tls_term.server_names.server_names')" variant="outlined"
                            :hint="$t('ports.config.tls_term.server_names.hint')" v-model="formData.tls_term.server_names"
                            density="compact" :rules="tlsServerNamesRules" persistent-hint></v-text-field>
                    </v-col>
                </v-row>
            </v-container>
        </div>
        <v-toolbar color="transparent" density="compact">
            <v-toolbar-title>
                {{ $t('ports.config.servers') }}
            </v-toolbar-title>
        </v-toolbar>
        <v-divider></v-divider>
        <v-container>
            <div v-for="(item, i) in formData.servers" :key="i">
                <v-row justify="end">
                    <v-col cols="12" sm="2">
                        <v-select :label="$t('ports.config.protocol')" :items="protocols" v-model="item.protocol"
                            variant="outlined" density="compact"></v-select>
                    </v-col>

                    <v-col cols="12" sm="5">
                        <v-text-field :label="$t('ports.config.host')" variant="outlined" density="compact"
                            v-model="item.host" :rules="serverNameRules"></v-text-field>
                    </v-col>

                    <v-col cols="12" sm="3">
                        <v-text-field :label="$t('ports.config.port')" v-model="item.port" required min="1" max="65535"
                            variant="outlined" density="compact" type="number" :rules="portRules"></v-text-field>
                    </v-col>

                    <v-col cols="12" sm="2">
                        <v-btn-group density="compact" class="float-right">
                            <v-btn :disabled="formData.servers.length <= 1" icon="mdi-minus"
                                @click="removeServer(i)"></v-btn>
                            <v-btn icon="mdi-plus" @click="insertServer(i)"></v-btn>
                        </v-btn-group>
                    </v-col>
                </v-row>
            </div>
        </v-container>
        <v-divider></v-divider>
        <v-card-actions class="justify-end">
            <v-btn v-if="!entry" :to="{ path: '/ports' }">
                {{ $t('ports.config.cancel') }}
            </v-btn>
            <v-btn v-if="entry" :loading="loading" type="submit" color="primary">
                {{ $t('ports.config.update') }}
            </v-btn>
            <v-btn v-else :loading="loading" type="submit" color="primary">
                {{ $t('ports.config.create') }}
            </v-btn>
        </v-card-actions>
    </v-form>
</template>
  
<script setup>
import { reactive, defineEmits, defineProps, onMounted } from 'vue';
import { Address6, Address4 } from 'ip-address';
import { useI18n } from 'vue-i18n'
import { isValidHostname, parseTlsServerNames } from '@/utils/validators'

const { t } = useI18n({ useScope: 'global' })

const emit = defineEmits(['submit', 'loading'])

const props = defineProps({
    entry: {
        type: Object,
        default: null,
    },
    loading: Boolean,
});

const formData = reactive({
    ifs: '0.0.0.0',
    port: 8080,
    protocol: 'tcp',
    servers: [{ host: 'example.com', port: 8080, protocol: 'tcp' }],
    tls_term: { server_names: '' }
});

const protocols = [
    { title: 'TCP', value: 'tcp' },
    { title: 'TLS', value: 'tls' }
];

onMounted(() => {
    if (props.entry) {
        const { host, port, protocol } = multiaddrToServer(props.entry.listen)
        formData.ifs = host
        formData.port = port
        formData.protocol = protocol
        formData.servers = props.entry.servers.map(s => multiaddrToServer(s.addr))
        const { tls_termination } = props.entry
        if (tls_termination) {
            formData.tls_term.server_names = tls_termination.server_names.join(', ')
        }
    }
})

async function submitForm(event) {
    let { valid } = await event;
    if (valid) {
        const entry = {
            listen: serverToMultiaddrWithProtocol(formData.protocol, formData.ifs, formData.port),
            servers: formData.servers.map(s => ({
                addr: serverToMultiaddrWithProtocol(s.protocol, s.host, s.port),
            }))
        }
        if (formData.protocol === 'tls') {
            entry.tls_termination = {
                server_names: parseTlsServerNames(formData.tls_term.server_names)
            }
        }
        emit('submit', entry);
    }
}

function removeServer(n) {
    formData.servers.splice(n, 1);
}

function insertServer(n) {
    formData.servers.splice(n + 1, 0, { url: '' });
}

function serverToMultiaddrWithProtocol(protocol, host, port) {
    if (protocol === 'tls') {
        return serverToMultiaddr(host, port) + '/tls'
    } else {
        return serverToMultiaddr(host, port)
    }
}

function serverToMultiaddr(host, port) {
    if (host.match(/^[0-9.]+$/)) {
        return `/ip4/${host}/tcp/${port}`
    }
    if (host.match(/^[0-9a-f:]+$/)) {
        return `/ip6/${host}/tcp/${port}`
    }
    return `/dns/${host}/tcp/${port}`
}

function multiaddrToServer(addr) {
    const ip4tls = addr.match(/\/ip4\/([0-9.]+)\/tcp\/([0-9]+)\/tls/)
    if (ip4tls) {
        return { host: ip4tls[1], port: ip4tls[2], protocol: 'tls' }
    }
    const ip6tls = addr.match(/\/ip6\/([0-9a-f:]+)\/tcp\/([0-9]+)\/tls/)
    if (ip6tls) {
        return { host: ip6tls[1], port: ip6tls[2], protocol: 'tls' }
    }
    const dnstls = addr.match(/\/dns(?:4|6)?\/([0-9a-z.-]+)\/tcp\/([0-9]+)\/tls/)
    if (dnstls) {
        return { host: dnstls[1], port: dnstls[2], protocol: 'tls' }
    }
    const ip4tcp = addr.match(/\/ip4\/([0-9.]+)\/tcp\/([0-9]+)/)
    if (ip4tcp) {
        return { host: ip4tcp[1], port: ip4tcp[2], protocol: 'tcp' }
    }
    const ip6tcp = addr.match(/\/ip6\/([0-9a-f:]+)\/tcp\/([0-9]+)/)
    if (ip6tcp) {
        return { host: ip6tcp[1], port: ip6tcp[2], protocol: 'tcp' }
    }
    const dnstcp = addr.match(/\/dns(?:4|6)?\/([0-9a-z.-]+)\/tcp\/([0-9]+)/)
    if (dnstcp) {
        return { host: dnstcp[1], port: dnstcp[2], protocol: 'tcp' }
    }
    return {}
}

const nameRules = [
    value => {
        if (value) return true
        return t('ports.config.rule.name_required')
    },
]

const interfaceRules = [
    value => {
        try {
            new Address4(value)
            return true
        } catch (_) { }
        try {
            new Address6(value)
            return true
        } catch (_) { }
        return t('ports.config.rule.interface_required')
    },
]

const serverNameRules = [
    value => {
        if (isValidHostname(value)) return true
        try {
            new Address4(value)
            return true
        } catch (_) { }
        try {
            new Address6(value)
            return true
        } catch (_) { }
        return t('ports.config.rule.hostname_required')
    },
]

const portRules = [
    value => {
        const number = parseInt(value, 10)
        if (1 <= number && number <= 65535) return true
        return t('ports.config.rule.port_required')
    },
]

const tlsServerNamesRules = [
    value => {
        const list = parseTlsServerNames(value)
        if (list.length > 0) return true
        return t('ports.config.tls_term.server_names.rule')
    },
]

</script>
  