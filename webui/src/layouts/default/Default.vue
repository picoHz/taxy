<template>
  <v-app>
    <v-layout>
      <v-navigation-drawer v-model="drawer">
        <v-list>
          <v-list-item prepend-icon="mdi-network" title="Ports" :to="{ path: '/ports' }"></v-list-item>
          <v-list-item prepend-icon="mdi-key-chain" title="Keyring" :to="{ path: '/keyring' }"></v-list-item>
          <v-list-item prepend-icon="mdi-code-json" append-icon="mdi-open-in-new" title="Swagger" href="/swagger-ui"
            target="_blank"></v-list-item>
        </v-list>
      </v-navigation-drawer>

      <v-main>
        <v-toolbar color="primary" dark extended flat>
          <v-app-bar-nav-icon @click="drawer = !drawer"></v-app-bar-nav-icon>
        </v-toolbar>

        <v-card class="mx-auto" max-width="700" style="margin-top: -64px;">
          <v-breadcrumbs :items="breadcrumbs">
            <template v-slot:divider>
              <v-icon icon="mdi-chevron-right"></v-icon>
            </template>
            <template v-slot:title="{ item }">
              <span v-if="item.trName" class="text-h6">{{ t(item.trName, item.trData) }}</span>
              <span v-else class="text-h6">{{ item.title }}</span>
            </template>
          </v-breadcrumbs>
          <router-view />
        </v-card>
      </v-main>
    </v-layout>
  </v-app>
</template>

<script setup>
import { ref } from 'vue';
import { computed } from 'vue';
import { useRoute } from 'vue-router';
import { useI18n } from 'vue-i18n'

const { t } = useI18n({ useScope: 'global' })

const route = useRoute();

const breadcrumbs = computed(() => {
  const { breadcrumb } = route.meta;
  if (breadcrumb) {
    return breadcrumb(route);
  } else {
    return [];
  }
});

const drawer = ref(undefined);
</script>
