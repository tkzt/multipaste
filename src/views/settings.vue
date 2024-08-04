<script setup lang="ts">
</script>

<template>
  <div class="h-100vh w-100vw flex flex-col rd-lg bg-transparent">
    <div class="flex grow flex-col p-2">
      <div class="tray-item card">
        <div class="text-sm dark:text-gray-100">
          开机自启动
        </div>
        <a-switch type="round" />
      </div>
      <div class="tray-item card mt-2">
        <div class="shrink-0 text-sm dark:text-gray-100">
          最大记录
        </div>
        <div class="box-border w-1/2 shrink-1">
          <a-input type="number" theme="dark" />
        </div>
      </div>
    </div>
  </div>
</template>

<style lang="css" scoped>
.tray-item {
  --at-apply: h-32px flex items-center justify-between cursor-default;
}
</style>
