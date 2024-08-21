<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { sendNotification } from '@tauri-apps/plugin-notification'
import { exit } from '@tauri-apps/plugin-process'
import { useDebounceFn } from '@vueuse/core'
import { onMounted, reactive, ref } from 'vue'

const config = reactive<Multipaste.Config>({
  max_items: 0,
  auto_start: false,
})
const transitionReady = ref(false)

onMounted(async () => {
  Object.assign(config, await invoke<Multipaste.Config>('get_config'))
  setTimeout(() => {
    transitionReady.value = true
  }, 400)
})

async function toggleAutoStart() {
  const updated = await invoke<boolean>('update_auto_start', {
    autoStart: !config.auto_start,
  })
  if (updated) {
    config.auto_start = !config.auto_start
  }
}

const updateMaxItems = useDebounceFn(async (event: Event) => {
  const { value } = event.target as HTMLInputElement
  const valueMaxItems = +value
  if (!valueMaxItems || valueMaxItems <= 0) {
    sendNotification({
      title: 'Warning',
      body: 'Invalid max items.',
    })
    return
  }
  const updated = await invoke<boolean>('update_max_items', {
    maxItems: valueMaxItems,
  })
  if (updated) {
    config.max_items = valueMaxItems
  }
})
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
            <input type="checkbox" :checked="config.auto_start" @input="toggleAutoStart">
            <span class="slider" :class="{ 'transition-ready': transitionReady }" />
          </label>
        </div>
      </div>
      <div class="tray-item mt-2 card">
        <div class="shrink-0 text-sm">
          最大记录
        </div>
        <div class="box-border w-1/2 shrink-1 overflow-hidden rounded-lg">
          <input
            :value="config.max_items"
            type="number"
            oninput="this.value = this.value.replace(/[^\d]/g, '');"
            class="box-border w-full border-none bg-white/20 p-2 text-gray-800 outline-none dark:bg-white/12"
            @input="updateMaxItems"
          >
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
