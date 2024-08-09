<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, onUpdated, ref, watchEffect } from 'vue'
import PerfectScrollbar from 'perfect-scrollbar'
import 'perfect-scrollbar/css/perfect-scrollbar.css'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import RecordItem from '../components/RecordItem.vue'

const itemsRef = ref<HTMLElement>()
const ps = ref<PerfectScrollbar>()

const items = ref<Multipaste.ClipboardRecord[]>([])

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

listen<Multipaste.ClipboardRecord[]>('fill-records', async (event) => {
  items.value = event.payload
})
</script>

<template>
  <div class="relative h-100vh flex flex-col overflow-hidden rd-lg">
    <div class="h-10 flex shrink-0 items-center justify-center text-lg" data-tauri-drag-region>
      <i class="i-mdi-drag-horizontal block cursor-grab dark:c-white" data-tauri-drag-region />
    </div>
    <div class="box-border w-full px-2 pb-1">
      <div class="box-border w-full flex rounded-lg bg-white/30 p-2 dark:bg-white/12">
        <i class="i-mdi-magnify mr-1 text-xl dark:text-gray-100" />
        <input
          class="shrink-1 grow-1 select-none border-none bg-transparent p-0 text-1rem outline-none placeholder-gray-600 dark:placeholder-gray-400"
          placeholder="Filter..."
        >
      </div>
    </div>
    <div
      ref="itemsRef" class="no-scrollbar relative box-border h-[calc(100%-.5rem)] select-none overflow-auto important:pa-2"
    >
      <div class="flex flex-col">
        <RecordItem v-for="item, index in items" :key="index" :item="item" :class="{ 'mt-2': index > 0 }" />
      </div>
    </div>
  </div>
</template>
