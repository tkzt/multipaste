<script setup lang="ts">
import { useMouseInElement } from '@vueuse/core'
import { ref } from 'vue'

defineProps<{
  item: string
}>()

const containerRef = ref<HTMLElement>()
const { isOutside: isOutsideContainer } = useMouseInElement(containerRef)
</script>

<template>
  <div
    ref="containerRef" class="relative box-border flex cursor-pointer items-center justify-between rd-2 bg-[rgba(255,255,255,.37)] pa-4 text-sm hover:bg-[rgba(255,255,255,.5)] dark:c-white"
  >
    <div>{{ item }}</div>
    <div v-if="!isOutsideContainer" class="absolute right-1 top-1 flex">
      <div class="btn">
        <i class="i-mdi-pin-outline rotate-45" />
      </div>
      <div class="btn">
        <i class="i-mdi-close" />
      </div>
    </div>
  </div>
</template>
