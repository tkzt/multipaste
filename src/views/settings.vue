<script setup lang="ts">
import { appWindow } from '@tauri-apps/api/window'
import { SwitchRoot, SwitchThumb } from 'radix-vue'

appWindow.listen('tauri://blur', async () => {
  await appWindow.hide()
})
</script>

<template>
  <div class="h-100vh w-100vw flex flex-col rd-lg bg-transparent">
    <div class="mt-2 flex grow flex-col p-2">
      <div class="flex justify-between rd-lg bg-white p-2">
        <div class="text-sm">
          开机自启动
        </div>
        <SwitchRoot
          class="relative h-[25px] w-[42px] flex cursor-default rounded-full bg-black/50 shadow-sm data-[state=checked]:bg-black focus-within:outline-black focus-within:outline"
        >
          <SwitchThumb
            class="my-auto block h-[21px] w-[21px] translate-x-0.5 rounded-full bg-white shadow-sm transition-transform duration-100 will-change-transform data-[state=checked]:translate-x-[19px]"
          />
        </SwitchRoot>
      </div>
      <div class="mt-2 flex items-center justify-between rd-lg bg-white p-2">
        <div class="shrink-0 text-sm">
          最大记录
        </div>
        <div class="box-border shrink-1 grow px-2">
          <input
            class="selection:bg-blackA9 h-[28px] w-full inline-flex appearance-none items-center justify-center rounded-[4px] border-none px-[10px] text-[15px] text-white leading-none shadow-[0_0_0_1px] outline-none outline-none selection:color-white focus:shadow-[0_0_0_2px_black]"
            type="number" value="Pedro Duarte"
          >
        </div>
      </div>
    </div>
  </div>
</template>
