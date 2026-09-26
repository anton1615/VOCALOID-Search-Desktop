import { watch, type Ref } from 'vue'
import type { PlaylistType, Video } from '../api/tauri-commands'
import type { PlayerCore } from './usePlayerCore'

interface PlayerPropSyncOptions {
  currentVideo: Readonly<Ref<Video | null>>
  currentVideoIndex: Readonly<Ref<number>>
  hasNext: Readonly<Ref<boolean>>
  playlistType: Readonly<Ref<PlaylistType>>
  playlistVersion: Readonly<Ref<number>>
  player: Pick<PlayerCore, 'handleVideoChange' | 'updateIndex' | 'updateHasNext'>
}

export function usePlayerPropSync({
  currentVideo,
  currentVideoIndex,
  hasNext,
  playlistType,
  playlistVersion,
  player,
}: PlayerPropSyncOptions): () => void {
  const stopIdentitySync = watch(
    () => [
      playlistType.value,
      playlistVersion.value,
      currentVideoIndex.value,
      currentVideo.value?.id ?? null,
    ] as const,
    async () => {
      await player.handleVideoChange(currentVideo.value, currentVideoIndex.value, hasNext.value)
    },
    { immediate: true },
  )
  const stopIndexSync = watch(currentVideoIndex, (index) => {
    player.updateIndex(index, hasNext.value)
  })
  const stopHasNextSync = watch(hasNext, (next) => {
    player.updateHasNext(next)
  })

  return () => {
    stopIdentitySync()
    stopIndexSync()
    stopHasNextSync()
  }
}
