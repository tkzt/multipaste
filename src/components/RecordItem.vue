<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { useMouseInElement } from '@vueuse/core'
import { onBeforeUnmount, ref } from 'vue'
import AsyncImage from './AsyncImage.vue'

defineProps<{
  item: Multipaste.ClipboardRecord
}>()
defineEmits(['pin', 'unpin', 'deleteRecord'])
const containerRef = ref<HTMLElement>()
const { isOutside: isOutsideContainer } = useMouseInElement(containerRef)

function truncateText(text: string) {
  if (text.length > 150) {
    return `${text.slice(0, 150)}...`
  }
  return text
}
</script>

<template>
  <div
    ref="containerRef" class="relative box-border flex cursor-pointer items-center justify-between pa-4 text-sm card"
    @click="invoke('copy_record', { id: item.id })"
  >
    <div class="w-full overflow-hidden">
      <template v-if="item.record_type === 'text'">
        {{ truncateText(item.record_value) }}
      </template>
      <suspense v-else>
        <template #fallback>
          Loading
        </template>
        <AsyncImage :url="item.record_value" />
      </suspense>
    </div>
    <div class="absolute right-1 top-1 flex">
      <div v-if="!isOutsideContainer" class="btn" @click.stop="$emit('deleteRecord', item.id)">
        <i-mdi-close />
      </div>
      <div
        v-if="!isOutsideContainer || item.pinned"
        class="btn"
        :class="{ 'bg-[rgba(0,0,0,.05)] dark:bg-[rgba(255,255,255,.25)]': item.pinned }"
        @click.stop="!item.pinned ? $emit('pin', item.id) : $emit('unpin', item.id)"
      >
        <i-mdi-pin-outline class="rotate-45" />
      </div>
    </div>
  </div>
</template>
