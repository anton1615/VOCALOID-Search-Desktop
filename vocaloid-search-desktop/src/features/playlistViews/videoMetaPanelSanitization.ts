const ALLOWED_DESCRIPTION_TAGS = new Set([
  'a',
  'b',
  'blockquote',
  'br',
  'code',
  'em',
  'i',
  'li',
  'ol',
  'p',
  'pre',
  'strong',
  'u',
  'ul',
])

const escapeHtml = (value: string) => value
  .replace(/&/g, '&amp;')
  .replace(/</g, '&lt;')
  .replace(/>/g, '&gt;')
  .replace(/"/g, '&quot;')
  .replace(/'/g, '&#39;')

export const sanitizeDescriptionLink = (href: string) => {
  const normalizedHref = href.trim()

  if (!normalizedHref) {
    return ''
  }

  try {
    const parsedUrl = new URL(normalizedHref, 'https://example.invalid')

    if (['http:', 'https:', 'mailto:'].includes(parsedUrl.protocol)) {
      return normalizedHref
    }
  } catch {
    return ''
  }

  return ''
}

const preserveTextLineBreaks = (value: string) => value.replace(/\r?\n/g, '<br>')

export const sanitizeDescriptionHtml = (html: string) => {
  if (typeof DOMParser === 'undefined') {
    return preserveTextLineBreaks(escapeHtml(html))
  }

  const parser = new DOMParser()
  const documentNode = parser.parseFromString(`<div>${html}</div>`, 'text/html')
  const container = documentNode.body.firstElementChild

  if (!container) {
    return ''
  }

  const sanitizeNode = (node: Node) => {
    if (node.nodeType === Node.COMMENT_NODE) {
      node.parentNode?.removeChild(node)
      return
    }

    if (node.nodeType !== Node.ELEMENT_NODE) {
      if ('textContent' in node && typeof node.textContent === 'string' && node.textContent.includes('\n')) {
        const textWithLineBreaks = preserveTextLineBreaks(escapeHtml(node.textContent))
        const fragment = documentNode.createDocumentFragment()
        const tempContainer = documentNode.createElement('div')
        tempContainer.innerHTML = textWithLineBreaks

        while (tempContainer.firstChild) {
          fragment.appendChild(tempContainer.firstChild)
        }

        node.parentNode?.replaceChild(fragment, node)
      }

      return
    }

    const element = node as HTMLElement
    const tagName = element.tagName.toLowerCase()

    if (!ALLOWED_DESCRIPTION_TAGS.has(tagName)) {
      const fragment = documentNode.createDocumentFragment()

      while (element.firstChild) {
        const child = element.firstChild
        element.removeChild(child)
        sanitizeNode(child)
        fragment.appendChild(child)
      }

      element.replaceWith(fragment)
      return
    }

    const rawHref = tagName === 'a' ? (element.getAttribute('href') ?? '') : ''

    for (const attribute of [...element.attributes]) {
      element.removeAttribute(attribute.name)
    }

    if (tagName === 'a') {
      const safeHref = sanitizeDescriptionLink(rawHref)

      if (safeHref) {
        element.setAttribute('href', safeHref)
        element.setAttribute('target', '_blank')
        element.setAttribute('rel', 'noopener noreferrer')
      }
    }

    for (const child of [...element.childNodes]) {
      sanitizeNode(child)
    }
  }

  for (const child of [...container.childNodes]) {
    sanitizeNode(child)
  }

  return container.innerHTML
}
