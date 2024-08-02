<script setup lang="ts">
import { appWindow } from '@tauri-apps/api/window'

appWindow.listen('tauri://blur', async () => {
  await appWindow.hide()
})
</script>

<template>
  <div class="h-100vh w-100vw flex flex-col rd-lg bg-transparent">
    <div class="flex grow flex-col p-2">
      <div class="h-32px flex items-center justify-between rd-lg bg-white/16 p-2">
        <div class="text-sm dark:text-gray-600">
          开机自启动
        </div>
        <a-switch type="round" />
      </div>
      <div class="mt-2 h-32px flex items-center justify-between rd-lg bg-white/16 p-2">
        <div class="shrink-0 text-sm dark:text-gray-600">
          最大记录
        </div>
        <div class="box-border w-1/2 shrink-1">
          <a-input type="number" />
        </div>
      </div>
    </div>
  </div>
</template>
