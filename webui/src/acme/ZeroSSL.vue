<template>
    <v-container>
        <v-row>
            <v-col cols="12" sm="12">
                <v-select :label="$t('acme.challenge')" :items="challenges" v-model="acmeChallange" variant="outlined"
                    density="compact"></v-select>
            </v-col>
            <v-col cols="12" sm="12">
                <v-text-field @update:modelValue="update" type="email" :label="$t('acme.email')" variant="outlined"
                    v-model="email" density="compact" :rules="emailRules" persistent-hint></v-text-field>
            </v-col>
            <v-col cols="12" sm="12">
                <v-text-field @update:modelValue="update" autocapitalize="off" :label="$t('acme.domain')" variant="outlined"
                    v-model="domain" density="compact" :rules="domainNameRules" persistent-hint></v-text-field>
            </v-col>
            <v-col cols="12" sm="12">
                <v-text-field @update:modelValue="update" autocapitalize="off" label="EAB KID" variant="outlined"
                    v-model="kid" density="compact" :rules="kidRules" persistent-hint></v-text-field>
            </v-col>
            <v-col cols="12" sm="12">
                <v-text-field @update:modelValue="update" autocapitalize="off" label="EAB HMAC Key" variant="outlined"
                    v-model="hmacKey" density="compact" :rules="hmacKeyRules" persistent-hint></v-text-field>
            </v-col>
        </v-row>
    </v-container>
</template>

<script setup>
import { ref, defineProps, defineEmits } from 'vue'
import { isValidHostname } from '@/utils/validators'
import { useI18n } from 'vue-i18n'

const { t } = useI18n({ useScope: 'global' })

const props = defineProps({
    modelValue: {
        type: Object,
        default: () => ({}),
    },
});

const emit = defineEmits(['update:modelValue']);

const update = (value) => {
    emit('update:modelValue', {
        challenge_type: acmeChallange.value,
        renewal_days: 60,
        identifiers: [
            value
        ],
        provider: "ZeroSSL",
        server_url: "https://acme.zerossl.com/v2/DV90",
        contacts: [`mailto:${email.value}`],
        is_trusted: true,
        eab: {
            key_id: kid.value,
            hmac_key: hmacKey.value,
        }
    })
}

const acmeChallange = ref('http-01');
const domain = ref('');
const email = ref('');
const kid = ref('');
const hmacKey = ref('');

const challenges = [
    { title: 'HTTP', value: 'http-01' }
]

const domainNameRules = [
    value => {
        if (isValidHostname(value)) return true
        return t('acme.add_acme.rule.hostname_required')
    },
]

const emailRules = [
    value => {
        if (/\S+@\S+\.\S+/.test(value)) return true
        return t('acme.add_acme.rule.email_required')
    },
]

const kidRules = [
    value => {
        if (value.length > 0) return true
        return t('acme.add_acme.rule.key_id_required')
    },
]

const hmacKeyRules = [
    value => {
        if (value.length > 0) return true
        return t('acme.add_acme.rule.hmac_key_required')
    },
]
</script>