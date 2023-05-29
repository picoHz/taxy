import { defineStore } from 'pinia'

export const useSitesStore = defineStore('sites', {
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