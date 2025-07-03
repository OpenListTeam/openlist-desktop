import { defineStore } from 'pinia'
import { ref } from 'vue'

import { TauriAPI } from '../api/tauri'

export const useRcloneStore = defineStore('rclone', () => {
  const loading = ref(false)
  const error = ref<string | undefined>()
  const serviceRunning = ref(false)

  function clearError() {
    error.value = undefined
  }

  async function startRcloneBackend() {
    try {
      loading.value = true
      const isRunning = await TauriAPI.rclone.backend.isRunning()
      if (isRunning) {
        serviceRunning.value = true
        return true
      }
      const result = await TauriAPI.rclone.backend.createAndStart()
      if (result) {
        serviceRunning.value = true
      }
      return result
    } catch (err: any) {
      error.value = 'Failed to start rclone service'
      throw err
    } finally {
      loading.value = false
    }
  }

  async function getRcloneProcessId() {
    try {
      const processList = await TauriAPI.process.list()
      const findRcloneBackend = processList.find(p => p.config?.name === 'single_rclone_backend_process')
      if (findRcloneBackend) {
        return findRcloneBackend.id
      }
    } catch (err) {
      console.error('Failed to get Rclone process ID from database:', err)
      return undefined
    }
  }

  async function stopRcloneBackend() {
    try {
      loading.value = true
      const id = await getRcloneProcessId()
      if (!id) {
        serviceRunning.value = false
        return
      }
      const result = await TauriAPI.process.stop(id)
      if (result) {
        serviceRunning.value = false
      }
      return result
    } catch (err: any) {
      error.value = 'Failed to stop rclone service'
      throw err
    } finally {
      loading.value = false
    }
  }

  async function checkRcloneBackendStatus() {
    try {
      const isRunning = await TauriAPI.rclone.backend.isRunning()
      serviceRunning.value = isRunning
      return isRunning
    } catch (err: any) {
      serviceRunning.value = false
      return false
    }
  }

  async function init() {
    console.log('Initializing Rclone store...')
  }

  return {
    // State
    loading,
    error,
    serviceRunning,
    clearError,
    startRcloneBackend,
    stopRcloneBackend,
    checkRcloneBackendStatus,
    init
  }
})
