import { register } from '@tauri-apps/plugin-global-shortcut'
import { invoke } from '@tauri-apps/api/core'

export default {
  install() {
    // register('Control+F', () => {
    //   invoke('search_focus')
    // })
    // register('Control+V', async () => {
    //   invoke('awake')
    // })
  },
}
