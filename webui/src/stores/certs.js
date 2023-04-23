import { defineStore } from 'pinia'

export const useCertsStore = defineStore('certs', {
    state: () => ({ list: {} }),
    actions: {
        update(certs) {
            this.list = certs
        },
    },
})