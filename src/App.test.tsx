import '@testing-library/jest-dom/vitest'
import { cleanup, fireEvent, render, screen } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'

const tauri = vi.hoisted(() => ({ invoke: vi.fn(), openUrl: vi.fn() }))
vi.mock('@tauri-apps/api/core', () => ({ invoke: tauri.invoke }))
vi.mock('@tauri-apps/plugin-opener', () => ({ openUrl: tauri.openUrl }))
import App from './App'

describe('WeChat Paste Bridge', () => {
  afterEach(cleanup)
  it('renders an import target and a disabled copy action', () => {
    render(<App />)

    expect(screen.getByText('导入文章包')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: '复制并打开微信编辑器' })).toBeDisabled()
  })

  it('shows validation risks and enables copy only for a valid package', () => {
    render(<App initialImport={{ title: '测试文章', canCopy: true, issues: ['assets/missing.jpg'] }} />)

    expect(screen.getByText('测试文章')).toBeInTheDocument()
    expect(screen.getByText('assets/missing.jpg')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: '复制并打开微信编辑器' })).toBeEnabled()
    expect(screen.getByRole('button', { name: '复制并打开秀米' })).toBeEnabled()
  })

  it('preserves the previous preview when clipboard writing fails', () => {
    render(<App initialImport={{ title: '测试文章', canCopy: true, issues: [] }} copyError="剪贴板暂时不可用，请关闭占用它的程序后重试。" />)

    expect(screen.getByText('测试文章')).toBeInTheDocument()
    expect(screen.getByRole('alert')).toHaveTextContent('剪贴板暂时不可用')
  })

  it('imports an article package through the supplied local bridge', async () => {
    const onImport = async () => ({ title: '导入后的文章', canCopy: true, issues: [] })
    render(<App onImport={onImport} />)

    fireEvent.click(screen.getByRole('button', { name: '选择文章包' }))

    expect(await screen.findByText('导入后的文章')).toBeInTheDocument()
  })

  it('reports a browser-opening error without mislabeling a successful copy as a clipboard failure', async () => {
    tauri.invoke.mockResolvedValueOnce(undefined)
    tauri.openUrl.mockRejectedValueOnce(new Error('浏览器启动失败'))
    render(<App initialImport={{ title: '测试文章', canCopy: true, issues: [], safeHtml: '<p>正文</p>', plainText: '正文' }} />)

    fireEvent.click(screen.getByRole('button', { name: '复制并打开微信编辑器' }))

    expect(await screen.findByRole('alert')).toHaveTextContent('已复制文章，但未能自动打开微信编辑器：浏览器启动失败')
  })

  it('shows the actual error when the native command rejects with a string', async () => {
    tauri.invoke.mockRejectedValueOnce('富文本格式回读失败：测试详情')
    render(<App initialImport={{ title: '测试文章', canCopy: true, issues: [], safeHtml: '<p>正文</p>', plainText: '正文' }} />)
    fireEvent.click(screen.getByRole('button', { name: '复制并打开秀米' }))
    expect(await screen.findByRole('alert')).toHaveTextContent('富文本格式回读失败：测试详情')
  })

  it('shows a success message when the copy and browser opening succeed', async () => {
    tauri.invoke.mockResolvedValueOnce(undefined)
    tauri.openUrl.mockResolvedValueOnce(undefined)
    render(<App initialImport={{ title: '测试文章', canCopy: true, issues: [], safeHtml: '<p>正文</p>', plainText: '正文' }} />)
    fireEvent.click(screen.getByRole('button', { name: '复制并打开秀米' }))
    expect(await screen.findByRole('status')).toHaveTextContent('已复制，进入秀米正文后按 Ctrl+V 粘贴')
  })
})
