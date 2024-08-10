<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { useMouseInElement } from '@vueuse/core'
import { ref } from 'vue'

const props = defineProps<{
  item: Multipaste.ClipboardRecord
}>()

const containerRef = ref<HTMLElement>()
const { isOutside: isOutsideContainer } = useMouseInElement(containerRef)

function copy() {
  invoke('copy_record', { id: props.item.id })
}
</script>

<template>
  <div
    ref="containerRef" class="relative box-border flex cursor-pointer items-center justify-between pa-4 text-sm card"
    @click="copy"
  >
    <div class="w-full overflow-hidden">
      {{ item.record_value }}
    </div>
    <div v-if="!isOutsideContainer" class="absolute right-1 top-1 flex">
      <div class="btn">
        <i-mdi-pin-outline class="rotate-45" />
      </div>
      <div class="btn">
        <i-mdi-close />
      </div>
    </div>
  </div>
</template>
