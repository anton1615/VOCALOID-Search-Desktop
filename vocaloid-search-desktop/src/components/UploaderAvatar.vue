<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import {
  createUploaderAvatarState,
  resolveUploaderAvatarStateAfterError,
} from '../features/playlistViews/uploaderAvatarState'

const props = withDefaults(defineProps<{
  src?: string | null
  alt?: string
  placeholderText?: string
}>(), {
  src: null,
  alt: 'Uploader avatar',
  placeholderText: '👤',
})

const currentSrc = ref<string | null>(null)
const showPlaceholder = ref(true)

function syncState(src: string | null) {
  const state = createUploaderAvatarState(src)
  currentSrc.value = state.currentSrc
  showPlaceholder.value = state.showPlaceholder
}

watch(() => props.src, (src) => {
  syncState(src)
}, { immediate: true })

function handleError() {
  const state = resolveUploaderAvatarStateAfterError({
    currentSrc: currentSrc.value,
    showPlaceholder: showPlaceholder.value,
  })
  currentSrc.value = state.currentSrc
  showPlaceholder.value = state.showPlaceholder
}

const placeholderClasses = computed(() => ({
  'default-avatar': showPlaceholder.value,
}))
</script>

<template>
  <img v-if="currentSrc" :src="currentSrc" :alt="alt" @error="handleError" />
  <div v-else :class="placeholderClasses">{{ placeholderText }}</div>
</template>
