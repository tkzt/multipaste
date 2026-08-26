<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { useMouseInElement } from '@vueuse/core'
import { ref } from 'vue'
import AsyncImage from './AsyncImage.vue'

const props = defineProps<{
  item: Multipaste.ClipboardRecord
  keyword: string
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

function highlightParts(text: string, keyword: string) {
  const target = truncateText(text)
  const query = keyword.trim()

  if (!query)
    return [{ text: target, isMatch: false }]

  const haystack = target.toLocaleLowerCase()
  const needle = query.toLocaleLowerCase()
  const parts: Array<{ text: string, isMatch: boolean }> = []
  let cursor = 0

  while (cursor < target.length) {
    const hit = haystack.indexOf(needle, cursor)
    if (hit === -1) {
      parts.push({ text: target.slice(cursor), isMatch: false })
      break
    }

    if (hit > cursor)
      parts.push({ text: target.slice(cursor, hit), isMatch: false })

    parts.push({ text: target.slice(hit, hit + query.length), isMatch: true })
    cursor = hit + query.length
  }

  return parts.length ? parts : [{ text: target, isMatch: false }]
}
</script>

<template>
  <div
    ref="containerRef" class="relative box-border flex cursor-pointer items-center justify-between pa-4 text-sm card"
    @click="invoke('copy_record', { id: item.id })"
  >
    <div class="w-full overflow-hidden">
      <template v-if="item.record_type === 'text'">
        <span
          v-for="(part, index) in highlightParts(item.record_value, props.keyword)"
          :key="index"
          :class="part.isMatch ? 'rounded bg-amber-200/70 px-0.5 dark:bg-amber-400/30' : ''"
        >
          {{ part.text }}
        </span>
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
