<template>
    <v-container>
        <v-row>
            <v-col cols="12" sm="12">
                <v-select :label="$t('certs.acme.challenge')" :items="challenges" v-model="acmeChallange" variant="outlined"
                    density="compact"></v-select>
            </v-col>
            <v-col cols="12" sm="12">
                <v-text-field @blur="handleBlur" :label="$t('certs.acme.domain')" variant="outlined" v-model="domain"
                    density="compact" :rules="domainNameRules" persistent-hint></v-text-field>
            </v-col>
        </v-row>
    </v-container>
</template>

<script setup>
import { ref, defineProps, defineEmits, computed } from 'vue'
import { isValidHostname } from '@/utils/validators'
import { useI18n } from 'vue-i18n'

const { t } = useI18n({ useScope: 'global' })

const props = defineProps({
    staging: {
        type: Boolean,
        default: false,
    },
    modelValue: {
        type: Object,
        default: () => ({}),
    },
});

const emit = defineEmits(['update:modelValue']);

const handleBlur = (event) => {
    emit('update:modelValue', {
        challenge_type: acmeChallange.value,
        identifiers: [
            domain.value
        ],
        provider: props.staging ? "Let's Encrypt (Staging)" : "Let's Encrypt",
        server_url: props.staging ? "https://acme-staging-v02.api.letsencrypt.org/directory" : "https://acme-v02.api.letsencrypt.org/directory"
    })
}

const acmeChallange = ref('http-01');
const domain = ref('');

const challenges = [
    { title: 'HTTP', value: 'http-01' }
]

const domainNameRules = [
    value => {
        if (isValidHostname(value)) return true
        return t('certs.acme.rule.hostname_required')
    },
]
</script>