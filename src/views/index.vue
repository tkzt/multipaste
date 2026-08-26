<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, ref, watch, watchEffect } from 'vue'
import PerfectScrollbar from 'perfect-scrollbar'
import 'perfect-scrollbar/css/perfect-scrollbar.css'
import { invoke } from '@tauri-apps/api/core'
import RecordItem from '../components/RecordItem.vue'

const itemsRef = ref<HTMLElement>()
const ps = ref<PerfectScrollbar>()

const items = ref<Multipaste.ClipboardRecord[]>([])
const keyword = ref('')
const isComposing = ref(false)
let latestQueryId = 0

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

onMounted(() => {
  filterRecords()
})

watch(keyword, filterRecords)

function onCompositionStart() {
  isComposing.value = true
}

function onCompositionEnd() {
  isComposing.value = false
  filterRecords()
}

async function filterRecords() {
  if (isComposing.value)
    return

  const queryId = ++latestQueryId
  const records = await invoke<Multipaste.ClipboardRecord[]>('filter_records', {
    keyword: keyword.value,
  })

  if (queryId !== latestQueryId)
    return

  items.value = records
}

async function pinRecord(id: number) {
  await invoke('pin_record', { id })
  await filterRecords()
}

async function unpinRecord(id: number) {
  await invoke('unpin_record', { id })
  await filterRecords()
}

async function deleteRecord(id: number) {
  await invoke('delete_record', { id })
  await filterRecords()
}
</script>

<template>
  <div class="relative h-100vh flex flex-col overflow-hidden rd-lg">
    <div class="h-10 flex shrink-0 cursor-grab items-center justify-center text-lg" data-tauri-drag-region>
      <i-mdi-drag-horizontal class="pointer-events-none block dark:c-white" />
    </div>
    <div class="box-border w-full px-2 pb-1">
      <div class="box-border w-full flex rounded-lg bg-white/30 p-2 dark:bg-white/12">
        <i-mdi-magnify class="mr-1 text-xl dark:text-gray-100" />
        <input
          v-model="keyword"
          @compositionstart="onCompositionStart"
          @compositionend="onCompositionEnd"
          class="shrink-1 grow-1 select-none border-none bg-transparent p-0 text-1rem outline-none dark:(text-gray-100 placeholder-gray-200) placeholder-gray-600"
          placeholder="Filter..."
        >
      </div>
    </div>
    <div
      ref="itemsRef" class="no-scrollbar relative box-border h-[calc(100%-.5rem)] select-none overflow-auto important:pa-2"
    >
      <div class="flex flex-col">
        <RecordItem v-for="item, index in items" :key="index" :item="item" :keyword="keyword" :class="{ 'mt-2': index > 0 }" @unpin="unpinRecord" @pin="pinRecord" @delete-record="deleteRecord" />
      </div>
    </div>
  </div>
</template>
