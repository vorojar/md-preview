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
const beforeSearchTop = await page.locator('#app').boundingBox();
await page.locator('#search-toggle').click();
const searchingTop = await page.locator('#app').boundingBox();
await page.locator('#search-input').fill('math');
await page.waitForSelector('mark.search-hit.current', { timeout: 1000 });
const searchHitCount = await page.locator('mark.search-hit').count();
await page.locator('#search-close').click();
const afterSearchTop = await page.locator('#app').boundingBox();
await page.locator('a[href^="javascript:"]').click();
await page.emulateMedia({ media: 'print' });

const result = await page.evaluate((searchHits) => ({
  title: document.getElementById('title').textContent,
  katex: document.querySelectorAll('.katex').length,
  mermaidSvg: document.querySelectorAll('.mdp-mermaid svg').length,
  topActionIcons: document.querySelectorAll('#top-actions .tool-button svg').length,
  searchHits,
  printTopbarDisplay: getComputedStyle(document.getElementById('topbar')).display,
  printSearchDisplay: getComputedStyle(document.getElementById('search-box')).display,
  printPreviewDisplay: getComputedStyle(document.getElementById('preview')).display,
  bad: window.__bad === 1
}), searchHitCount);

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
