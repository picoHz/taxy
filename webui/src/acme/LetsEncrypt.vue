<template>
    <v-container>
        <v-row>
            <v-col cols="12" sm="12">
                <v-select :label="$t('certs.acme.challenge')" :items="challenges" v-model="acmeChallange" variant="outlined"
                    density="compact"></v-select>
            </v-col>
            <v-col cols="12" sm="12">
                <v-text-field type="email" :label="$t('certs.acme.email')" variant="outlined" v-model="emailAdress"
                    density="compact" persistent-hint></v-text-field>
            </v-col>
            <v-col cols="12" sm="12">
                <v-text-field :label="$t('certs.acme.domain')" variant="outlined" v-model="domain" density="compact"
                    persistent-hint></v-text-field>
            </v-col>
        </v-row>
        <DnsChallenge></DnsChallenge>
    </v-container>
</template>

<script setup>
import { ref } from 'vue'
import DnsChallenge from '@/acme/DnsChallenge.vue';

const acmeChallange = ref('dns-01');
const emailAdress = ref('');
const domain = ref('');

const challenges = [
    { title: 'DNS', value: 'dns-01' }
]
</script>