export type PostMessageTarget = {
  postMessage: (message: unknown, targetOrigin: string) => void
}

export function rememberPlayerMessageSource(source: MessageEventSource | null): PostMessageTarget | null {
  if (!source || typeof source !== 'object' || !('postMessage' in source)) {
    return null
  }

  return source as PostMessageTarget
}

export function clearPlayerMessageSource(_source: PostMessageTarget | null): PostMessageTarget | null {
  return null
}
