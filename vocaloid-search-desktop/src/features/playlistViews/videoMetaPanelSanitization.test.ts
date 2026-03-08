import { afterEach, describe, expect, test, vi } from 'vitest'
import {
  sanitizeDescriptionHtml,
  sanitizeDescriptionLink,
} from './videoMetaPanelSanitization'

describe('VideoMetaPanel description sanitization', () => {
  afterEach(() => {
    vi.unstubAllGlobals()
  })

  test('removes unsafe attributes and disallowed protocols while preserving safe links', () => {
    class TestCommentNode {}
    class TestTextNode {
      constructor(public textContent: string) {}
    }

    class TestElement {
      private attributeMap = new Map<string, string>()
      public childNodes: Array<TestElement | TestTextNode | TestCommentNode> = []
      public parentNode: TestElement | TestDocumentFragment | null = null

      constructor(
        public tagName: string,
        attributes: Record<string, string> = {},
        childNodes: Array<TestElement | TestTextNode | TestCommentNode> = [],
      ) {
        for (const [name, value] of Object.entries(attributes)) {
          this.attributeMap.set(name, value)
        }
        for (const child of childNodes) {
          this.appendChild(child)
        }
      }

      get nodeType() {
        return 1
      }

      get firstChild() {
        return this.childNodes[0] ?? null
      }

      get attributes() {
        return [...this.attributeMap.entries()].map(([name, value]) => ({ name, value }))
      }

      getAttribute(name: string) {
        return this.attributeMap.get(name) ?? null
      }

      removeAttribute(name: string) {
        this.attributeMap.delete(name)
      }

      setAttribute(name: string, value: string) {
        this.attributeMap.set(name, value)
      }

      removeChild(child: TestElement | TestTextNode | TestCommentNode) {
        const index = this.childNodes.indexOf(child)
        if (index >= 0) {
          this.childNodes.splice(index, 1)
          if (child instanceof TestElement || child instanceof TestTextNode || child instanceof TestCommentNode) {
            ;(child as { parentNode: TestElement | TestDocumentFragment | null }).parentNode = null
          }
        }
        return child
      }

      appendChild(child: TestElement | TestTextNode | TestCommentNode) {
        if (child instanceof TestElement || child instanceof TestTextNode || child instanceof TestCommentNode) {
          ;(child as { parentNode: TestElement | TestDocumentFragment | null }).parentNode = this
        }
        this.childNodes.push(child)
        return child
      }

      replaceWith(fragment: TestDocumentFragment) {
        const parent = this.parentNode
        if (!parent) {
          return
        }

        const siblings = parent.childNodes
        const index = siblings.indexOf(this)
        if (index < 0) {
          return
        }

        siblings.splice(index, 1, ...fragment.childNodes)
        for (const child of fragment.childNodes) {
          ;(child as { parentNode: TestElement | TestDocumentFragment | null }).parentNode = parent
        }
        fragment.childNodes = []
        this.parentNode = null
      }

      get innerHTML() {
        return this.childNodes.map(renderNode).join('')
      }

      get attributesList() {
        return [...this.attributes.entries()].map(([name, value]) => ({ name, value }))
      }
    }

    class TestDocumentFragment {
      public childNodes: Array<TestElement | TestTextNode | TestCommentNode> = []
      public parentNode: TestElement | TestDocumentFragment | null = null

      appendChild(child: TestElement | TestTextNode | TestCommentNode) {
        ;(child as { parentNode: TestElement | TestDocumentFragment | null }).parentNode = this
        this.childNodes.push(child)
        return child
      }
    }

    class TestDocument {
      constructor(public body: { firstElementChild: TestElement | null }) {}

      createDocumentFragment() {
        return new TestDocumentFragment()
      }
    }

    const renderNode = (node: TestElement | TestTextNode | TestCommentNode): string => {
      if (node instanceof TestCommentNode) {
        return ''
      }

      if (node instanceof TestTextNode) {
        return node.textContent
      }

      const attrs = [...node.attributes.map(({ name, value }) => [name, value] as const)]
        .map(([name, value]) => ` ${name}="${value}"`)
        .join('')

      return `<${node.tagName}${attrs}>${node.childNodes.map(renderNode).join('')}</${node.tagName}>`
    }

    const safeHttpsLink = new TestElement('a', {
      href: 'https://example.com/watch?v=1',
      onclick: 'alert(1)',
      style: 'color:red',
    }, [new TestTextNode('safe')])
    const safeMailtoLink = new TestElement('a', {
      href: 'mailto:test@example.com',
      onmouseover: 'alert(2)',
    }, [new TestTextNode('mail')])
    const unsafeLink = new TestElement('a', {
      href: 'javascript:alert(1)',
      onclick: 'alert(3)',
    }, [new TestTextNode('bad')])
    const strippedWrapper = new TestElement('div', { class: 'wrapper' }, [
      new TestElement('strong', { onclick: 'alert(4)' }, [new TestTextNode('nested')]),
    ])
    const container = new TestElement('div', {}, [
      safeHttpsLink,
      new TestTextNode(' '),
      safeMailtoLink,
      new TestTextNode(' '),
      unsafeLink,
      new TestTextNode(' '),
      strippedWrapper,
      new TestCommentNode(),
    ])

    class FakeDOMParser {
      parseFromString() {
        return new TestDocument({ firstElementChild: container })
      }
    }

    vi.stubGlobal('DOMParser', FakeDOMParser)
    vi.stubGlobal('Node', {
      ELEMENT_NODE: 1,
      COMMENT_NODE: 8,
    })

    expect(sanitizeDescriptionHtml('ignored')).toBe(
      '<a href="https://example.com/watch?v=1" target="_blank" rel="noopener noreferrer">safe</a> ' +
      '<a href="mailto:test@example.com" target="_blank" rel="noopener noreferrer">mail</a> ' +
      '<a>bad</a> <strong>nested</strong>',
    )
  })

  test('allows only expected link protocols', () => {
    expect(sanitizeDescriptionLink('https://example.com')).toBe('https://example.com')
    expect(sanitizeDescriptionLink('mailto:test@example.com')).toBe('mailto:test@example.com')
    expect(sanitizeDescriptionLink('javascript:alert(1)')).toBe('')
    expect(sanitizeDescriptionLink('data:text/html,<b>x</b>')).toBe('')
  })

  test('preserves plain text line breaks with DOMParser available', () => {
    expect(sanitizeDescriptionHtml('first line\nsecond line')).toBe('first line<br>second line')
  })

  test('falls back to escaped text with line breaks when DOMParser is unavailable', () => {
    vi.stubGlobal('DOMParser', undefined)

    expect(sanitizeDescriptionHtml('<script>alert(1)</script>\n<a href="javascript:alert(2)">x</a>')).toBe(
      '&lt;script&gt;alert(1)&lt;/script&gt;<br>&lt;a href=&quot;javascript:alert(2)&quot;&gt;x&lt;/a&gt;',
    )
  })
})
