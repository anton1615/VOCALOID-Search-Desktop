import { defineStore } from 'pinia'
import { ref } from 'vue'
import { api, type BlockedUploader, type UploaderCandidate } from '../api/tauri-commands'

const SEARCH_DEBOUNCE_MS = 300

export const useUploaderBlacklistStore = defineStore('uploaderBlacklist', () => {
  const isOpen = ref(false)
  const items = ref<BlockedUploader[]>([])
  const loading = ref(false)
  const candidates = ref<UploaderCandidate[]>([])
  const searching = ref(false)
  const keyword = ref('')

  let debounceTimer: number | null = null
  // Monotonic request id: a slow response may only apply if no newer search started.
  let searchRequestId = 0
  let initialLoadStarted = false

  async function refresh() {
    loading.value = true
    try {
      items.value = await api.getUploaderBlacklist()
    } catch (error) {
      console.error('[uploaderBlacklist] Failed to load blacklist:', error)
    } finally {
      loading.value = false
    }
  }

  function openDialog() {
    isOpen.value = true
  }

  function closeDialog() {
    isOpen.value = false
    cancelPendingSearch()
  }

  function cancelPendingSearch() {
    if (debounceTimer !== null) {
      window.clearTimeout(debounceTimer)
      debounceTimer = null
    }
  }

  async function runSearch(trimmed: string, requestId: number) {
    searching.value = true
    try {
      const result = await api.searchUploaders(trimmed)
      if (requestId !== searchRequestId) return
      candidates.value = result
    } catch (error) {
      if (requestId !== searchRequestId) return
      candidates.value = []
      console.error('[uploaderBlacklist] Failed to search uploaders:', error)
    } finally {
      if (requestId === searchRequestId) {
        searching.value = false
      }
    }
  }

  function searchCandidates(rawKeyword: string) {
    keyword.value = rawKeyword
    cancelPendingSearch()

    const trimmed = rawKeyword.trim()
    // Bump immediately so any in-flight response can no longer overwrite newer input.
    const requestId = ++searchRequestId

    if (trimmed.length === 0) {
      candidates.value = []
      searching.value = false
      return
    }

    debounceTimer = window.setTimeout(() => {
      debounceTimer = null
      void runSearch(trimmed, requestId)
    }, SEARCH_DEBOUNCE_MS)
  }

  async function add(uploaderId: string, displayName?: string | null) {
    items.value = await api.addUploaderToBlacklist(uploaderId, displayName ?? null)
    return items.value
  }

  async function remove(uploaderId: string) {
    items.value = await api.removeUploaderFromBlacklist(uploaderId)
    return items.value
  }

  function isBlocked(uploaderId: string): boolean {
    return items.value.some((item) => item.uploader_id === uploaderId)
  }

  function ensureInitialLoad() {
    if (initialLoadStarted) return
    initialLoadStarted = true
    void refresh()
  }

  ensureInitialLoad()

  return {
    isOpen,
    items,
    loading,
    candidates,
    searching,
    keyword,
    openDialog,
    closeDialog,
    refresh,
    searchCandidates,
    add,
    remove,
    isBlocked,
  }
})
