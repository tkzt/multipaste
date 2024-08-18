<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { exit } from '@tauri-apps/plugin-process'
import { onMounted, ref } from 'vue'

const config = ref<Multipaste.Config>()
const transitionReady = ref(false)

onMounted(async () => {
  config.value = await invoke<Multipaste.Config>('get_config')
  setTimeout(() => {
    transitionReady.value = true
  }, 400)
})

async function toggleAutoStart() {
  if (config.value) {
    const updated = await invoke<boolean>('update_auto_start', {
      autoStart: !config.value.auto_start,
    })
    if (updated) {
      config.value.auto_start = !config.value.auto_start
    }
  }
}

// TODO: Debounce
// async function updateMaxItems() {
//   if (config.value && config.value.max_items) {
//     const updated = await invoke<boolean>('update_max_items', {
//       max_items: config.value.max_items,
//     })
//     if (updated) {
//       config.value.max_items = !config.value.max_items
//     }
//   }
// }
</script>

<template>
  <div class="h-100vh w-100vw flex flex-col rd-lg bg-transparent">
    <div class="flex grow flex-col p-2">
      <div class="tray-item card">
        <div class="text-sm">
          开机自启动
        </div>
        <div class="box-border w-1/2 flex shrink-1 items-center justify-end overflow-hidden rounded-lg">
          <label class="switch">
            <input type="checkbox" :checked="config?.auto_start" @input="toggleAutoStart">
            <span class="slider" :class="{ 'transition-ready': transitionReady }" />
          </label>
        </div>
      </div>
      <div class="tray-item mt-2 card">
        <div class="shrink-0 text-sm">
          最大记录
        </div>
        <div class="box-border w-1/2 shrink-1 overflow-hidden rounded-lg">
          <input type="number" oninput="this.value = this.value.replace(/[^\d]/g, '');" :value="config?.max_items" class="border-none bg-white/20 p-2 text-gray-800 outline-none dark:bg-white/12">
        </div>
      </div>
      <div class="mt-2 cursor-pointer bg-red-600 text-center text-sm text-white hover:(bg-red-700 dark:bg-red-500) card" @click="exit(0)">
        退出
      </div>
    </div>
  </div>
</template>

<style lang="css" scoped>
.tray-item {
  --at-apply: h-32px flex items-center justify-between cursor-default;
}

.transition-ready {
  --at-apply: transition-400;
}

.transition-ready::before {
  --at-apply: transition-400;
}

.slider {
  --at-apply: absolute left-0 right-0 top-0 bottom-0 rd-34px bg-white/20 dark:
    bg-white/12;
}

.slider::before {
  --at-apply: "absolute bottom-4px left-4px content-[''] w-16px h-16px rd-50% bg-slate-200 dark:bg-slate-300";
}

.switch {
  --at-apply: relative inline-block w-40px h-24px cursor-pointer;
}

.switch input {
  --at-apply: hidden;
}

.switch input:checked + .slider {
  --at-apply: bg-slate-900;
}

.switch input:checked + .slider::before {
  --at-apply: translate-x-16px;
}
</style>
