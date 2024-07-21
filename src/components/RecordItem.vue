<template>
  <div class="
    box-border pa-4 bg-[rgba(255,255,255,.37)] rd-2 hover:bg-[rgba(255,255,255,.5)]
    dark:c-white text-sm flex items-center justify-between relative cursor-pointer
  " ref="containerRef">
    <div>{{ item }}</div>
    <div class="flex absolute right-1 top-1" v-if="!isOutsideContainer">
      <div class="btn">
        <i class="i-mdi-pin-outline rotate-45"></i>
      </div>
      <div class="btn">
        <i class="i-mdi-close"></i>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useMouseInElement } from "@vueuse/core"
import { ref } from "vue"
defineProps<{
  item: string
}>()

const containerRef = ref<HTMLElement>()
const { isOutside: isOutsideContainer } = useMouseInElement(containerRef)
</script>