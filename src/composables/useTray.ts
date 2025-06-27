import { onMounted, onUnmounted } from 'vue'

import { TauriAPI } from '../api/tauri'
import { useAppStore } from '../stores/app'
import { useCoreActions } from './useCoreActions'

export const useTray = () => {
  const { startOpenListCore, stopOpenListCore, restartOpenListCore } = useCoreActions()
  const store = useAppStore()

  let unlistenTrayActions: (() => void) | null = null
  const updateTrayMenu = async (serviceRunning: boolean) => {
    try {
      await TauriAPI.updateTrayMenuDelayed(serviceRunning)
    } catch (error) {
      console.error('Failed to update tray menu:', error)
    }
  }

  const handleTrayServiceAction = async (action: string) => {
    try {
      switch (action) {
        case 'start':
          await startOpenListCore()
          setTimeout(async () => {
            await updateTrayMenu(store.openlistCoreStatus.running)
          }, 5000)
          break
        case 'stop':
          await stopOpenListCore()
          setTimeout(async () => {
            await updateTrayMenu(store.openlistCoreStatus.running)
          }, 5000)
          break
        case 'restart':
          await restartOpenListCore()
          setTimeout(async () => {
            await updateTrayMenu(store.openlistCoreStatus.running)
          }, 5000)
          break
        default:
          console.warn('Unknown tray service action:', action)
      }
    } catch (error) {
      console.error(`Failed to execute tray action '${action}':`, error)
      setTimeout(async () => {
        await updateTrayMenu(store.openlistCoreStatus.running)
      }, 3000)
    }
  }
  const initTrayListeners = async () => {
    try {
      unlistenTrayActions = await TauriAPI.listenToTrayServiceActions(handleTrayServiceAction)

      await TauriAPI.forceUpdateTrayMenu(store.openlistCoreStatus.running)
      console.log('Tray listeners initialized and menu updated')
    } catch (error) {
      console.error('Failed to initialize tray listeners:', error)
    }
  }

  const cleanupTrayListeners = () => {
    if (unlistenTrayActions) {
      unlistenTrayActions()
      unlistenTrayActions = null
    }
  }

  onMounted(() => {
    initTrayListeners()
  })

  onUnmounted(() => {
    cleanupTrayListeners()
  })

  return {
    updateTrayMenu,
    initTrayListeners,
    cleanupTrayListeners
  }
}
