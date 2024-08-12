<script setup lang="ts">
import { convertFileSrc } from '@tauri-apps/api/core'

const props = defineProps<{
  url: string
}>()

async function convertImgUrl(url: string) {
  return convertFileSrc(url)
}
const urlConverted = await convertImgUrl(props.url)
</script>

<template>
  <div class="w-full">
    <img :src="urlConverted" class="inline-block max-h-37 w-full object-contain">
  </div>
</template>
