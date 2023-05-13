import { defineStore } from 'pinia'

export const useAcmeStore = defineStore('acme', {
    state: () => ({ list: [] }),
    actions: {
        update(list) {
            this.list = list
        },
    },
    getters: {
        getStatusbyId: (state) => {
            return (id) => state.list.find((item) => item.id === id)
        },
    }
})