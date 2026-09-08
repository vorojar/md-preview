import { createRequire } from 'module';
import { readFile } from 'node:fs/promises';

const require = createRequire(import.meta.url);
const { chromium } = require('playwright');

const root = new URL('../..', import.meta.url);
const preview = new URL('mobile/shared/preview.html', root).href;
const previewCss = await readFile(new URL('mobile/shared/mobile-preview.css', root), 'utf8');
const browser = await chromium.launch({ headless: true });
const page = await browser.newPage({
  viewport: { width: 390, height: 844 },
  deviceScaleFactor: 2,
  isMobile: true
});

const errors = [];
page.on('pageerror', error => errors.push(error.message));
page.on('console', message => {
  if (message.type() === 'error') errors.push(message.text());
});

await page.goto(preview);
await page.waitForLoadState('domcontentloaded');
await page.evaluate(() => {
  window.MDPreview.render({
    name: 'mobile-fixture.md',
    baseHref: 'file:///tmp/md-preview-docs/',
    markdown: [
      '# Mobile fixture',
      '',
      'Inline math $a^2+b^2=c^2$ and display math:',
      '',
      '$$E=mc^2$$',
      '',
      '> [!IMPORTANT]',
      '> This alert should render as a GitHub alert.',
      '',
      'This has ==highlighted text== but `==literal code==` stays literal.',
      '',
      '```mermaid',
      'graph TD',
      '  A[Open] --> B[Preview]',
      '```',
      '',
      '[bad](javascript:window.__bad=1)'
    ].join('\n')
  });
});

await page.waitForSelector('.katex', { timeout: 5000 });
await page.waitForSelector('.mdp-mermaid svg', { timeout: 5000 });
await page.waitForSelector('.markdown-alert-important', { timeout: 1000 });
await page.waitForSelector('mark.mdp-mark', { timeout: 1000 });
const beforeSearchTop = await page.locator('#app').boundingBox();
await page.locator('#search-toggle').click();
const searchingTop = await page.locator('#app').boundingBox();
await page.locator('#search-input').fill('math');
await page.waitForSelector('mark.search-hit.current', { timeout: 1000 });
const searchHitCount = await page.locator('mark.search-hit').count();
await page.locator('#search-close').click();
const afterSearchTop = await page.locator('#app').boundingBox();
await page.locator('a[href^="javascript:"]').click();
await page.emulateMedia({ colorScheme: 'dark' });
const darkAlert = await page.evaluate(() => ({
  background: getComputedStyle(document.querySelector('.markdown-alert-important')).backgroundColor,
  titleColor: getComputedStyle(document.querySelector('.markdown-alert-important .markdown-alert-title')).color
}));
await page.emulateMedia({ media: 'print', colorScheme: 'dark' });

const result = await page.evaluate((searchHits) => ({
  title: document.getElementById('title').textContent,
  katex: document.querySelectorAll('.katex').length,
  mermaidSvg: document.querySelectorAll('.mdp-mermaid svg').length,
  alertTitle: document.querySelector('.markdown-alert-important .markdown-alert-title')?.textContent.trim(),
  alertText: document.querySelector('.markdown-alert-important p:not(.markdown-alert-title)')?.textContent.trim(),
  alertBg: getComputedStyle(document.querySelector('.markdown-alert-important')).backgroundColor,
  highlightText: document.querySelector('mark.mdp-mark')?.textContent,
  codeLiteral: document.querySelector('code')?.textContent,
  topActionIcons: document.querySelectorAll('#top-actions .tool-button svg').length,
  searchHits,
  printTopbarDisplay: getComputedStyle(document.getElementById('topbar')).display,
  printSearchDisplay: getComputedStyle(document.getElementById('search-box')).display,
  printPreviewDisplay: getComputedStyle(document.getElementById('preview')).display,
  bad: window.__bad === 1
}), searchHitCount);

await page.emulateMedia({ media: 'screen' });
await page.evaluate(() => {
  window.__positions = {};
  window.MDPreviewAndroid = {
    getReadingProgress: id => window.__positions[id] || 0,
    saveReadingProgress: (id, progress) => { window.__positions[id] = progress; }
  };
  window.__readingFixture = { documentId: 'content://provider/document/A', name: 'A.md', markdown: '# Reading\n\n' + ('A long paragraph for reading.\n\n'.repeat(200)) };
  window.MDPreview.render(window.__readingFixture);
});
await page.waitForTimeout(100);
if (await page.evaluate(() => window.scrollY) !== 0) throw new Error('New document must start at top');
await page.evaluate(() => {
  window.dispatchEvent(new Event('wheel'));
  window.scrollTo(0, (document.documentElement.scrollHeight - innerHeight) * 0.6);
});
await page.waitForTimeout(100);
const progress = await page.evaluate(() => window.__positions['content://provider/document/A']);
if (Math.abs(progress - 0.6) > 0.02) throw new Error(`Reading progress not saved: ${progress}`);
await page.evaluate(() => window.MDPreview.render({ ...window.__readingFixture, documentId: 'content://provider/document/B' }));
await page.waitForTimeout(100);
if (await page.evaluate(() => window.scrollY) !== 0) throw new Error('Different files share reading progress');
await page.evaluate(() => window.MDPreview.render(window.__readingFixture));
await page.waitForTimeout(100);
const restored = await page.evaluate(() => window.scrollY / (document.documentElement.scrollHeight - innerHeight));
if (Math.abs(restored - 0.6) > 0.02) throw new Error(`Reading progress not restored: ${restored}`);
await page.evaluate(() => {
  const block = document.createElement('div'); block.style.height = '1000px';
  document.getElementById('preview').append(block);
});
await page.waitForTimeout(100);
const delayed = await page.evaluate(() => window.scrollY / (document.documentElement.scrollHeight - innerHeight));
if (Math.abs(delayed - 0.6) > 0.02) throw new Error(`Delayed layout lost reading progress: ${delayed}`);
await browser.close();

if (errors.length) {
  throw new Error(`Renderer console errors:\n${errors.join('\n')}`);
}
if (result.title !== 'mobile-fixture.md') {
  throw new Error(`Unexpected title: ${result.title}`);
}
if (!result.katex || !result.mermaidSvg || !result.searchHits) {
  throw new Error(`Renderer feature check failed: ${JSON.stringify(result)}`);
}
if (result.alertTitle !== 'Important' ||
    result.alertText !== 'This alert should render as a GitHub alert.' ||
    result.alertBg === 'rgba(0, 0, 0, 0)' ||
    result.highlightText !== 'highlighted text' ||
    result.codeLiteral !== '==literal code==') {
  throw new Error(`Markdown extension check failed: ${JSON.stringify(result)}`);
}
if (darkAlert.background !== 'rgb(22, 27, 34)' ||
    darkAlert.titleColor !== 'rgb(163, 113, 247)') {
  throw new Error(`Dark alert style check failed: ${JSON.stringify(darkAlert)}`);
}
if (result.topActionIcons !== 3) {
  throw new Error(`Toolbar icons missing: ${JSON.stringify(result)}`);
}
if (Math.abs(beforeSearchTop.y - searchingTop.y) > 1 ||
    Math.abs(beforeSearchTop.y - afterSearchTop.y) > 1) {
  throw new Error(`Search changed document position: ${JSON.stringify({
    before: beforeSearchTop.y,
    searching: searchingTop.y,
    after: afterSearchTop.y
  })}`);
}
if (result.printTopbarDisplay !== 'none' ||
    result.printSearchDisplay !== 'none' ||
    result.printPreviewDisplay === 'none' ||
    !/@page\s*{\s*margin:\s*12mm;\s*}/.test(previewCss)) {
  throw new Error(`Print stylesheet check failed: ${JSON.stringify(result)}`);
}
if (result.bad) {
  throw new Error('javascript: link executed');
}

console.log('[mobile-renderer] OK');
