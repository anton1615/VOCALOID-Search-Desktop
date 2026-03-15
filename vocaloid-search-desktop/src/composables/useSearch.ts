import { ref, type Ref } from 'vue'
import { api, type Video } from '../api/tauri-commands'
import { buildSearchRequest, type SearchPersistenceState } from '../features/playlistViews/searchViewState'

export interface UseSearchOptions {
  pageSize?: number
}

export interface UseSearchReturn {
  query: Ref<string>
  loading: Ref<boolean>
  results: Ref<Video[]>
  totalCount: Ref<number>
  page: Ref<number>
  pageSize: Ref<number>
  hasNext: Ref<boolean>
  loadingMore: Ref<boolean>
  search: (filters: SearchPersistenceState) => Promise<void>
  loadMore: () => Promise<void>
  clearResults: () => void
}

export function useSearch(options: UseSearchOptions = {}): UseSearchReturn {
  const { pageSize: initialPageSize = 50 } = options

  const query = ref('')
  const loading = ref(false)
  const results = ref<Video[]>([]) as Ref<Video[]>
  const totalCount = ref(0)
  const page = ref(1)
  const pageSize = ref(initialPageSize)
  const hasNext = ref(false)
  const loadingMore = ref(false)

  async function search(filters: SearchPersistenceState): Promise<void> {
    loading.value = true
    page.value = 1
    results.value = []

    try {
      const response = await api.search(buildSearchRequest({
        query: query.value,
        page: page.value,
        pageSize: pageSize.value,
        state: filters,
      }))
      results.value = response.results
      totalCount.value = response.total
      hasNext.value = response.has_next
    } catch (e) {
      console.error('Search error:', e)
    } finally {
      loading.value = false
    }
  }

  async function loadMore(): Promise<void> {
    if (loadingMore.value || !hasNext.value) return

    loadingMore.value = true

    try {
      const searchState = await api.getSearchState()
      const response = await api.loadMore('Search', searchState.version)
      results.value = [...results.value, ...response.results]
      page.value++
      hasNext.value = response.has_next
    } catch (e) {
      console.error('Load more error:', e)
    } finally {
      loadingMore.value = false
    }
  }

  function clearResults(): void {
    results.value = []
    totalCount.value = 0
    page.value = 1
    hasNext.value = false
  }

  return {
    query,
    loading,
    results,
    totalCount,
    page,
    pageSize,
    hasNext,
    loadingMore,
    search,
    loadMore,
    clearResults,
  }
}
