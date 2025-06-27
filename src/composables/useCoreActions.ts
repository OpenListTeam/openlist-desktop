import { useAppStore } from '../stores/app'

export const useCoreActions = () => {
  const store = useAppStore()

  const startOpenListCore = async () => {
    try {
      await store.startOpenListCore()
    } catch (error) {
      console.error('Failed to start service:', error)
      throw error
    }
  }

  const stopOpenListCore = async () => {
    try {
      await store.stopOpenListCore()
    } catch (error) {
      console.error('Failed to stop service:', error)
      throw error
    }
  }

  const restartOpenListCore = async () => {
    try {
      await store.restartOpenListCore()
    } catch (error) {
      console.error('Failed to restart service:', error)
      throw error
    }
  }

  return {
    startOpenListCore,
    stopOpenListCore,
    restartOpenListCore
  }
}
