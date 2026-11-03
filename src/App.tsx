import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { openUrl } from '@tauri-apps/plugin-opener'
import { compileForTarget } from './lib/paste'

export type ImportPresentation = { title: string; canCopy: boolean; issues: string[]; safeHtml?: string; plainText?: string; wechatHtml?: string; xiumiHtml?: string }
type AppProps = { initialImport?: ImportPresentation; copyError?: string; onImport?: () => Promise<ImportPresentation> }

async function importFromDesktop(): Promise<ImportPresentation> {
  const selected = await open({ directory: true, multiple: false, title: '选择包含 manifest.json 的文章包文件夹' })
  if (!selected || Array.isArray(selected)) throw new Error('未选择文章包。')
  return invoke<ImportPresentation>('import_article_package', { path: selected })
}

export default function App({ initialImport, copyError, onImport }: AppProps) {
  const [imported, setImported] = useState<ImportPresentation | undefined>(initialImport)
  const [error, setError] = useState(copyError)
  const [status, setStatus] = useState<string>()
  const handleImport = async () => {
    setError(undefined)
    setStatus(undefined)
    try { setImported(await (onImport ?? importFromDesktop)()) }
    catch (reason) { setError(`导入失败：${reason instanceof Error ? reason.message : String(reason)}`) }
  }
  const handleCopy = async (target: 'wechat' | 'xiumi') => {
    if (!imported?.safeHtml || !imported.plainText) return
    setError(undefined)
    setStatus(undefined)
    try {
      const source = target === 'wechat' ? (imported.wechatHtml ?? imported.safeHtml) : (imported.xiumiHtml ?? imported.safeHtml)
      const editable = compileForTarget(source, target)
      await invoke('copy_article', { document: { safe_html: editable.safeHtml, plain_text: editable.plainText, images: [], editable_blocks: 0, fixed_components: [] } })
      try {
        await openUrl(target === 'wechat' ? 'https://mp.weixin.qq.com/' : 'https://xiumi.us/')
        setStatus(`已复制，进入${target === 'wechat' ? '微信编辑器' : '秀米'}正文后按 Ctrl+V 粘贴。`)
      } catch (reason) {
        const editor = target === 'wechat' ? '微信编辑器' : '秀米'
        setError(`已复制文章，但未能自动打开${editor}：${reason instanceof Error ? reason.message : String(reason)}`)
      }
    }
    catch (reason) { setError(reason instanceof Error ? reason.message : (typeof reason === 'string' ? reason : '复制未确认成功，请重试。')) }
  }
  return (
    <main className="shell">
      <header><p className="eyebrow">WECHAT PASTE BRIDGE</p><h1>导入文章包，一键复制到公众号</h1><p>Codex 负责出稿，软件负责检查、兼容化和本地复制。</p></header>
      <section className="import-card" aria-label="导入文章包"><h2>导入文章包</h2><p>选择包含 manifest.json、article.html 和 assets 文件夹的文章包。</p><button type="button" onClick={handleImport}>选择文章包</button></section>
      <section className="status-card" aria-label="导入状态"><h2>{imported ? imported.title : '等待导入'}</h2>{imported ? <><p>文章包已载入。请查看下方自检结果。</p><ul>{imported.issues.map((issue) => <li key={issue}>{issue}</li>)}</ul></> : <p>导入后会显示公众号宽度预览、素材检查与复制风险。</p>}</section>
      {error && <p role="alert">{error}</p>}
      {status && <p role="status">{status}</p>}
      <section className="status-card" aria-label="复制目标"><h2>选择编辑器</h2><p>软件会先复制兼容版内容，再打开目标编辑器。进入正文编辑区后按 Ctrl+V。</p><button type="button" disabled={!imported?.canCopy} onClick={() => handleCopy('wechat')}>复制并打开微信编辑器</button><button type="button" disabled={!imported?.canCopy} onClick={() => handleCopy('xiumi')}>复制并打开秀米</button></section>
    </main>
  )
}
