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
await page.addInitScript(() => {
  window.__readingPositionSaves = [];
  window.MDPreviewAndroid = {
    getRecent() {},
    saveReadingPosition(documentKey, progress) {
      window.__readingPositionSaves.push({ documentKey, progress });
    }
  };
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

await page.emulateMedia({ media: 'screen', colorScheme: 'light' });
const readingMarkdown = Array.from(
  { length: 140 },
  (_, index) => `## Reading section ${index + 1}\n\nParagraph ${index + 1} keeps the fixture scrollable.`
).join('\n\n');
await page.evaluate(markdown => {
  window.MDPreview.render({
    name: 'reading-position.md',
    documentKey: 'uri:content://fixture/reading-position.md',
    readingProgress: 0.62,
    markdown
  });
}, readingMarkdown);
await page.waitForFunction(() => {
  const maxScroll = document.documentElement.scrollHeight - window.innerHeight;
  return maxScroll > 0 && Math.abs(window.scrollY / maxScroll - 0.62) < 0.025;
});
await page.evaluate(() => {
  const extra = document.createElement('div');
  extra.innerHTML = Array.from(
    { length: 20 },
    (_, index) => `<h2>Deferred section ${index + 1}</h2><p>Late rendered content.</p>`
  ).join('');
  document.getElementById('preview').appendChild(extra);
});
await page.waitForFunction(() => {
  const maxScroll = document.documentElement.scrollHeight - window.innerHeight;
  return maxScroll > 0 && Math.abs(window.scrollY / maxScroll - 0.62) < 0.025;
});
await page.evaluate(() => {
  window.dispatchEvent(new PointerEvent('pointerdown'));
  const maxScroll = document.documentElement.scrollHeight - window.innerHeight;
  window.scrollTo(0, maxScroll * 0.37);
});
await page.waitForTimeout(300);
const savedReadingPosition = await page.evaluate(() => {
  const saves = window.__readingPositionSaves;
  return saves[saves.length - 1];
});
await page.evaluate(({ markdown, progress }) => {
  window.scrollTo(0, 0);
  window.MDPreview.render({
    name: 'reading-position.md',
    documentKey: 'uri:content://fixture/reading-position.md',
    readingProgress: progress,
    markdown
  });
}, { markdown: readingMarkdown, progress: savedReadingPosition.progress });
await page.waitForFunction(expected => {
  const maxScroll = document.documentElement.scrollHeight - window.innerHeight;
  return maxScroll > 0 && Math.abs(window.scrollY / maxScroll - expected) < 0.025;
}, savedReadingPosition.progress);
const reopenedReadingProgress = await page.evaluate(() => {
  const maxScroll = document.documentElement.scrollHeight - window.innerHeight;
  return maxScroll ? window.scrollY / maxScroll : 0;
});
await page.evaluate(() => {
  window.MDPreview.render({
    name: 'short.md',
    documentKey: 'uri:content://fixture/short.md',
    readingProgress: 0.8,
    markdown: '# Short document'
  });
  window.MDPreview.flushReadingPosition();
});
const shortDocument = await page.evaluate(() => ({
  scrollY: window.scrollY,
  saved: window.__readingPositionSaves[window.__readingPositionSaves.length - 1]
}));

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
if (savedReadingPosition.documentKey !== 'uri:content://fixture/reading-position.md' ||
    Math.abs(savedReadingPosition.progress - 0.37) > 0.025 ||
    Math.abs(reopenedReadingProgress - savedReadingPosition.progress) > 0.025) {
  throw new Error(`Reading position persistence failed: ${JSON.stringify({
    savedReadingPosition,
    reopenedReadingProgress
  })}`);
}
if (shortDocument.scrollY !== 0 ||
    shortDocument.saved.documentKey !== 'uri:content://fixture/short.md' ||
    shortDocument.saved.progress !== 0) {
  throw new Error(`Short-document reading position failed: ${JSON.stringify(shortDocument)}`);
}

console.log('[mobile-renderer] OK');
