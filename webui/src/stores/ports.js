import { defineStore } from 'pinia'

export const usePortsStore = defineStore('ports', {
  state: () => ({ table: [], status: {} }),
  actions: {
    updateTable(table) {
      this.table = table
    },
    updateStatus(id, status) {
      this.status[id] = status
    },
  },
  getters: {
    getStateByName: (state) => {
      return (id) => {
        const status = state.status[id]
        if (!status) return 'unknown';
        const { socket, tls } = status.state;
        if (socket !== 'listening') return socket;
        if (tls && tls !== 'active') return tls;
        return socket;
      }
    },
    getStatusByName: (state) => {
      return (id) => state.status[id] || {}
    },
  }
})