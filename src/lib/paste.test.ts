import { describe, expect, it } from 'vitest'
import { buildLockedComponentHtml } from './paste'
import { compileForTarget, makeEditableWeChatHtml } from './paste'

describe('buildLockedComponentHtml', () => {
  it('creates paste-safe image blocks without the source web layout', () => {
    const html = buildLockedComponentHtml(['data:image/png;base64,AAA', 'data:image/png;base64,BBB'])

    expect(html).toContain('data:image/png;base64,AAA')
    expect(html).toContain('data:image/png;base64,BBB')
    expect(html).not.toContain('position:absolute')
    expect(html).not.toContain('<table')
  })
})

describe('compileForTarget', () => {
  const source = '<section><img src="data:image/png;base64,AAA" style="position:absolute"><table style="border:1px solid #fff"><tr><td>可编辑内容</td></tr></table></section>'

  it('uses different target-specific structures and keeps the copy editable', () => {
    const wechat = compileForTarget(source, 'wechat').safeHtml
    const xiumi = compileForTarget(source, 'xiumi').safeHtml

    expect(wechat).toContain('可编辑内容')
    expect(xiumi).toContain('可编辑内容')
    expect(wechat).not.toContain('data:image/png;base64,AAA')
    expect(wechat).not.toContain('<table')
    expect(xiumi).toContain('<table')
    expect(wechat).not.toBe(xiumi)
  })

  it('keeps Xiumi sections and three editable column widths in the outgoing HTML', () => {
    const source = '<section style="background-color:#151816"><section data-stage-columns="true"><section style="display:inline-block;width:33.33%"><p>录制前</p></section><section style="display:inline-block;width:33.33%"><p>录制中</p></section><section style="display:inline-block;width:33.33%"><p>录制后</p></section></section></section>'
    const output = compileForTarget(source, 'xiumi').safeHtml
    const doc = new DOMParser().parseFromString(output, 'text/html')
    expect(doc.querySelector('section')?.style.backgroundColor).toBe('rgb(21, 24, 22)')
    expect(Array.from(doc.querySelectorAll<HTMLElement>('[data-stage-columns] > section'), (e) => e.style.width)).toEqual(['33.33%', '33.33%', '33.33%'])
    expect(doc.body.textContent).toContain('录制中')
  })
})

describe('makeEditableWeChatHtml', () => {
  it('keeps every text node editable while removing layout rules WeChat rewrites', () => {
    const output = makeEditableWeChatHtml('<section style="position:relative;background:#151816"><p style="font-size:18px">可编辑标题</p><img src="data:image/png;base64,AAA" style="position:absolute;width:50%;height:200px"></section>')

    expect(output.safeHtml).toContain('可编辑标题')
    expect(output.safeHtml).not.toContain('<img')
    expect(output.safeHtml).not.toContain('position:')
    expect(output.safeHtml).not.toContain('height:200px')
    expect(output.safeHtml).not.toContain('border')
    expect(output.safeHtml).toContain('color: rgb(244, 242, 235)')
    expect(output.plainText).toContain('可编辑标题')
  })

  it('keeps a styled section wrapper so WeChat can retain a section background', () => {
    const output = makeEditableWeChatHtml('<section style="background:#151816;padding:20px"><p>可编辑标题</p></section>')

    expect(output.safeHtml).toContain('<section')
    expect(output.safeHtml).toContain('background:#151816')
    expect(output.safeHtml).toContain('可编辑标题')
  })

  it('removes whitespace nodes between top-level components', () => {
    const output = makeEditableWeChatHtml('<section style="background:#151816">甲</section>\n<section style="background:#20241f">乙</section>')

    expect(output.safeHtml).toBe('<section style="background:#151816">甲</section><section style="background:#20241f">乙</section>')
  })

  it('preserves editable three-column widths in designated stage columns', () => {
    const source = '<section><section data-stage-columns="true" style="font-size:0"><section style="display:inline-block;width:33.33%;vertical-align:top"><p>01 录制前</p></section><section style="display:inline-block;width:33.33%;vertical-align:top"><p>02 录制中</p></section><section style="display:inline-block;width:33.33%;vertical-align:top"><p>03 录制后</p></section></section></section>'
    const doc = new DOMParser().parseFromString(makeEditableWeChatHtml(source).safeHtml, 'text/html')
    const columns = doc.querySelectorAll<HTMLElement>('[data-stage-columns] > section')
    expect(columns).toHaveLength(3)
    expect(Array.from(columns, (column) => column.style.width)).toEqual(['33.33%', '33.33%', '33.33%'])
    expect(columns[0].textContent).toContain('01 录制前')
    expect(columns[2].textContent).toContain('03 录制后')
  })
})
