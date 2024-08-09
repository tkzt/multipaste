<script setup lang="ts">
import { exit } from '@tauri-apps/plugin-process'
</script>

<template>
  <div class="h-100vh w-100vw flex flex-col rd-lg bg-transparent">
    <div class="flex grow flex-col p-2">
      <div class="tray-item card">
        <div class="text-sm">
          开机自启动
        </div>
        <a-switch type="round" disabled />
      </div>
      <div class="tray-item mt-2 card">
        <div class="shrink-0 text-sm">
          最大记录
        </div>
        <div class="box-border w-1/2 shrink-1">
          <a-input type="number" theme="dark" disabled />
        </div>
      </div>
      <div class="mt-3 cursor-pointer text-center text-sm text-red-600" @click="exit(0)">
        退出
      </div>
    </div>
  </div>
</template>

<style lang="css" scoped>
.tray-item {
  --at-apply: h-32px flex items-center justify-between cursor-default;
}
</style>
