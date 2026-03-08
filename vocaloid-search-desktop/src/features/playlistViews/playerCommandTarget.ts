type PostMessageTarget = {
  postMessage: (message: unknown, targetOrigin: string) => void
}

interface ResolvePlayerCommandTargetOptions {
  lastMessageSource: PostMessageTarget | null
  iframeWindow: PostMessageTarget | null
}

export function resolvePlayerCommandTarget({
  lastMessageSource,
  iframeWindow,
}: ResolvePlayerCommandTargetOptions): PostMessageTarget | null {
  return lastMessageSource ?? iframeWindow ?? null
}
