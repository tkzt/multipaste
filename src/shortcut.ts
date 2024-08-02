import { register } from '@tauri-apps/api/globalShortcut'
import { invoke } from '@tauri-apps/api'

export default {
  install() {
    register('Control+F', () => {
      invoke('search_focus')
    })
    register('Control+V', () => {
      invoke('awake')
    })
  },
}
