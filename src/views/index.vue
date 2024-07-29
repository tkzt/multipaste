<script setup lang="ts">
import { nextTick, onBeforeUnmount, ref, watchEffect } from 'vue';
import { useDark } from '@vueuse/core';
import PerfectScrollbar from 'perfect-scrollbar';
import 'perfect-scrollbar/css/perfect-scrollbar.css';
import RecordItem from '../components/RecordItem.vue';


const itemsRef = ref<HTMLElement>()
const ps = ref<PerfectScrollbar>()

const items = ref(['Nisi ipsum eiusmod cillum eu dolore est cupidatat ea aute laboris eu do.', 'Nisi ipsum eiusmod cillum eu dolore est cupidatat ea aute laboris eu do.', 'yyy', 'xxx', 'yyy', 'xxx', 'yyy', 'xxx', 'yyy', 'Nisi ipsum eiusmod cillum eu dolore est cupidatat ea aute laboris eu do.', 'Nisi ipsum eiusmod cillum eu dolore est cupidatat ea aute laboris eu do.', 'yyy', 'xxx', 'yyy', 'xxx', 'yyy', 'xxx', 'yyy'])

watchEffect(() => {
  if (itemsRef.value && !ps.value) {
    nextTick(() => {
      ps.value = new PerfectScrollbar(itemsRef.value!)
    })
  }
})

onBeforeUnmount(() => {
  ps.value?.destroy()
})
</script>

<template>
  <div class="relative rd-3 overflow-hidden flex flex-col h-100vh">
    <div class="h-10 text-lg shrink-0 flex items-center justify-center" data-tauri-drag-region>
      <i class="i-mdi-drag-horizontal block cursor-grab dark:c-white" data-tauri-drag-region></i>
    </div>
    <div class="w-full px-2 box-border pb-1">
      <div class="w-full rounded-lg bg-white/12 p-2 box-border flex">
        <i class="i-mdi-magnify text-xl mr-1"></i>
        <input
          class="outline-none border-none bg-transparent p-0 text-1rem shrink-1 grow-1 select-none"
          placeholder="Filter..." />
      </div>
    </div>
    <div class="
      important:pa-2 box-border select-none relative overflow-auto
      h-[calc(100%-.5rem)] no-scrollbar
    " ref="itemsRef">
      <div class="flex flex-col">
        <RecordItem :item="item" :class="{ 'mt-2': index > 0 }" v-for="item, index in items" />
      </div>
    </div>
  </div>
</template>