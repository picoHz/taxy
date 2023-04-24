import { defineStore } from 'pinia'

export const useCertsStore = defineStore('certs', {
    state: () => ({ list: {} }),
    actions: {
        update(certs) {
            this.list = certs
        },
    },
    getters: {
        getStatusbyId: (state) => {
            return (id) => state.list.find((item) => item.id === id)
        },
    }
})