import { defineStore } from 'pinia'

export const useConfigStore = defineStore('config', {
    state: () => ({ app: {} }),
    actions: {
        update(config) {
            this.app = config
        },
    },
})