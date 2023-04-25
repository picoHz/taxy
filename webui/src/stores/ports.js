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
    getStateByName: (state) => {
      return (name) => {
        const status = state.status[name]
        if (!status) return 'unknown';
        const { socket, tls } = status.state;
        if (socket !== 'listening') return socket;
        if (tls && tls !== 'active') return tls;
        return socket;
      }
    },
    getStatusByName: (state) => {
      return (name) => state.status[name] || {}
    },
  }
})