<template>
  <div class="h-100vh w-100vw bg-slate-100 flex flex-col">
    <div class="grow p-2 flex flex-col mt-2">
      <div class="bg-white flex justify-between p-2 rd-lg">
        <div class="text-sm">开机自启动</div>
        <SwitchRoot
          class="w-[42px] h-[25px] focus-within:outline focus-within:outline-black flex bg-black/50 shadow-sm rounded-full relative data-[state=checked]:bg-black cursor-default">
          <SwitchThumb
            class="block w-[21px] h-[21px] my-auto bg-white shadow-sm rounded-full transition-transform duration-100 translate-x-0.5 will-change-transform data-[state=checked]:translate-x-[19px]">
          </SwitchThumb>
        </SwitchRoot>
      </div>
      <div class="bg-white flex justify-between items-center p-2 rd-lg mt-2">
        <div class="text-sm shrink-0">最大记录</div>
        <div class="box-border px-2 grow shrink-1">
          <input
            class="outline-none border-none inline-flex h-[28px] w-full appearance-none items-center justify-center rounded-[4px] px-[10px] text-[15px] leading-none text-white shadow-[0_0_0_1px] outline-none focus:shadow-[0_0_0_2px_black] selection:color-white selection:bg-blackA9"
            type="number" value="Pedro Duarte">
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { appWindow } from '@tauri-apps/api/window';
import { Fn, useTimeoutFn } from '@vueuse/core';
import { SwitchRoot, SwitchThumb } from 'radix-vue'
// import { ref } from 'vue';

// const stopHiding = ref<Fn>()

// appWindow.listen('tauri://focus', () => {
//   stopHiding.value?.()
//   stopHiding.value = undefined
// })

useTimeoutFn(async () => {
  appWindow.listen("tauri://blur", async () => {
    // await appWindow.hide()
  })
}, 3000)
</script>