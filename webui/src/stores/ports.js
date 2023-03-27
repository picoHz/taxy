import { defineStore } from 'pinia'

export const usePortsStore = defineStore('ports', {
  state: () => ({ table: [], status: {} }),
  actions: {
    updateTable(table) {
      this.table = table
    },
    updateStatus(name, status) {
      this.status[name] = status
    },
  },
  getters: {
    getStatusbyName: (state) => {
      return (name) => state.status[name] || {}
    },
  }
})