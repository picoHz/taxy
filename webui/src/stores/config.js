import { defineStore } from 'pinia'

export const useConfigStore = defineStore('config', {
    state: () => ({ app: {} }),
    actions: {
        update(config) {
            this.app = config
        },
        setCertSearchPaths(paths) {
            console.log(JSON.stringify(this.app));
            this.app.certs.search_paths = paths
        }
    },
})