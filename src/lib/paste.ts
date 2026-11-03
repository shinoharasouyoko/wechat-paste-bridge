export function buildLockedComponentHtml(images: string[]): string {
  return images.map((source) => `<p style="margin:0;"><img src="${source}" style="display:block;width:100%;height:auto;" /></p>`).join('')
}

export function makeEditableWeChatHtml(sourceHtml: string): { safeHtml: string; plainText: string } {
  const parsed = new DOMParser().parseFromString(sourceHtml, 'text/html')
  return sanitize(parsed, 'wechat')
}

export function compileForTarget(sourceHtml: string, target: 'wechat' | 'xiumi'): { safeHtml: string; plainText: string } {
  const parsed = new DOMParser().parseFromString(sourceHtml, 'text/html')
  return sanitize(parsed, target)
}

function sanitize(parsed: Document, target: 'wechat' | 'xiumi'): { safeHtml: string; plainText: string } {
  Array.from(parsed.body.childNodes).forEach((node) => {
    if (node.nodeType === Node.TEXT_NODE && !node.textContent?.trim()) node.remove()
  })
  parsed.querySelectorAll('section').forEach((section) => {
    Array.from(section.childNodes).forEach((node) => {
      if (node.nodeType === Node.TEXT_NODE && !node.textContent?.trim()) node.remove()
    })
  })
  parsed.querySelectorAll('script').forEach((element) => element.remove())
  parsed.querySelectorAll<HTMLElement>('img[style*="position:absolute"], img[style*="position: absolute"]').forEach((image) => {
    if (!(target === 'wechat' && image.hasAttribute('data-paste-background-layer'))) image.remove()
  })
  const unsupported = ['position', 'z-index', 'overflow', 'object-fit', 'object-position', 'max-width', 'width', 'height', 'table-layout', 'border-collapse', 'border', 'border-left', 'border-right', 'border-top', 'border-bottom']
  parsed.querySelectorAll<HTMLElement>('[style]').forEach((element) => {
    unsupported.forEach((property) => {
      if (target === 'wechat' && (element.hasAttribute('data-paste-background-layer') || element.hasAttribute('data-paste-background-container') || element.hasAttribute('data-paste-background-content')) && ['position', 'z-index', 'overflow', 'width', 'height'].includes(property)) return
      if (property === 'width' && element.parentElement?.hasAttribute('data-stage-columns')) return
      element.style.removeProperty(property)
    })
  })
  parsed.querySelectorAll('img').forEach((image) => {
    if (target === 'wechat' && image.hasAttribute('data-paste-background-layer')) return
    image.setAttribute('style', 'display:block;width:100%;height:auto;margin:0;')
  })
  parsed.querySelectorAll<HTMLElement>('p,h1,h2,h3,h4,h5,h6,li,td,th,blockquote,span').forEach((element) => {
    if (!element.style.color) element.style.color = '#f4f2eb'
  })
  if (target === 'wechat') {
    parsed.querySelectorAll('table').forEach((table) => {
      const replacement = document.createElement('div')
      replacement.style.margin = '16px 0'
      table.querySelectorAll('td,th').forEach((cell) => {
        const paragraph = document.createElement('p')
        paragraph.style.margin = '0 0 8px'
        paragraph.style.color = '#f4f2eb'
        paragraph.innerHTML = cell.innerHTML
        replacement.appendChild(paragraph)
      })
      table.replaceWith(replacement)
    })
  } else {
    parsed.querySelectorAll<HTMLElement>('table').forEach((table) => {
      table.style.width = '100%'
      table.style.border = '0'
    })
  }
  return {
    safeHtml: parsed.body.innerHTML,
    plainText: parsed.body.textContent?.replace(/\s+/g, ' ').trim() ?? '',
  }
}

