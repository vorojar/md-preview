import { createRequire } from 'node:module';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';

const require = createRequire(import.meta.url);
const { chromium } = require('playwright');

const root = resolve(new URL('..', import.meta.url).pathname);
const mainRs = await readFile(resolve(root, 'src/main.rs'), 'utf8');
const marker = 'var ICON_EDIT';
const markerIndex = mainRs.indexOf(marker);
if (markerIndex < 0) throw new Error('desktop script marker not found');
const scriptStart = mainRs.lastIndexOf('<script>', markerIndex);
const scriptEnd = mainRs.indexOf('</script>', markerIndex);
if (scriptStart < 0 || scriptEnd < 0) throw new Error('desktop script block not found');

const layoutCss = mainRs.match(/#app \{\{[^\n]+/)[0].replaceAll("{{", "{").replaceAll("}}", "}");
const desktopScript = mainRs
  .slice(scriptStart + '<script>'.length, scriptEnd)
  .replaceAll('{{', '{')
  .replaceAll('}}', '}')
  .replaceAll('{stat_words_js}', '字')
  .replaceAll('{stat_chars_js}', '字符');

const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 900, height: 700 } });
await page.route('https://md-preview.test/**', (route) => route.fulfill({
  contentType: 'text/html',
  body: '<!doctype html><title>MD Preview test</title>',
}));
await page.goto('https://md-preview.test/');
const previewBlocks = Array.from(
  { length: 100 },
  (_, index) => `<p>Preview paragraph ${index + 1}</p>`,
).join('');
const editorText = Array.from(
  { length: 280 },
  (_, index) => `Editor source line ${index + 1}`,
).join('\n');

await page.setContent(`<!doctype html>
<html>
  <head>
    <meta charset="utf-8">
    <style>
      ${layoutCss}
      :root { --content-scale: 1; }
      body { margin: 0; font: 15px/1.6 system-ui, sans-serif; }
      #preview { font-size: calc(15px * var(--content-scale)); }
      #editor {
        display: none;
        width: 100%;
        box-sizing: border-box;
        overflow: hidden;
        resize: none;
        font: calc(14px * var(--content-scale))/1.6 monospace;
      }
      body.editing #preview { display: none; }
      body.editing #editor { display: block; }
      .zoom-popover { display: none; }
      .zoom-control.open .zoom-popover { display: flex; }
      .toolbar { position: fixed; top: 0; right: 0; z-index: 10; }
      .toolbar button { width: 34px; height: 34px; }
    </style>
  </head>
  <body class="has-tabs">
    <div class="tabbar">
      <div id="tabs"></div>
      <div id="doc-stats"></div>
      <button id="tab-open"></button>
    </div>
    <div class="toolbar">
      <button id="btn-open"></button>
      <button id="btn-search"></button>
      <button id="btn-toggle"></button>
      <button id="btn-print"></button>
      <div id="zoom-control" class="zoom-control">
        <button id="btn-zoom"></button>
        <div class="zoom-popover">
          <button id="btn-zoom-out"></button>
          <button id="btn-zoom-reset"></button>
          <button id="btn-zoom-in"></button>
        </div>
      </div>
      <button id="btn-update" hidden></button>
    </div>
    <div class="findbar">
      <input id="find-input">
      <span id="find-state"></span>
      <button id="find-prev"></button>
      <button id="find-next"></button>
      <button id="find-close"></button>
    </div>
    <div id="app">
      <div id="preview">${previewBlocks}</div>
      <textarea id="editor">${editorText}</textarea>
    </div>
    <script>
      window.__messages = [];
      window.ipc = { postMessage(message) { window.__messages.push(message); } };
    </script>
    <script>${desktopScript}</script>
  </body>
</html>`);

await page.evaluate(() => window.__setContent(
  document.getElementById('preview').innerHTML,
  '你好 A\n',
  '',
  false,
  false,
));

let result = await page.evaluate(() => ({
  stats: document.getElementById('doc-stats').textContent,
  scale: getComputedStyle(document.documentElement).getPropertyValue('--content-scale').trim(),
  toolbarWidth: document.getElementById('btn-open').getBoundingClientRect().width,
}));
if (result.stats !== '3 字 · 5 字符' || result.scale !== '1' || result.toolbarWidth !== 34) {
  throw new Error(`initial reading tools failed: ${JSON.stringify(result)}`);
}

await page.locator('#btn-toggle').click();
await page.locator('#editor').fill('你 好');
result = await page.evaluate(() => ({
  stats: document.getElementById('doc-stats').textContent,
  dirty: window.__messages.includes('dirty:1'),
}));
if (result.stats !== '2 字 · 3 字符' || !result.dirty) {
  throw new Error(`live stats failed: ${JSON.stringify(result)}`);
}
await page.locator('#btn-toggle').click();

await page.locator('#btn-zoom').click();
await page.locator('#btn-zoom-in').click();
result = await page.evaluate(() => ({
  resetLabel: document.getElementById('btn-zoom-reset').textContent,
  scale: getComputedStyle(document.documentElement).getPropertyValue('--content-scale').trim(),
  stored: localStorage.getItem('md-preview-content-zoom-v1'),
  toolbarWidth: document.getElementById('btn-open').getBoundingClientRect().width,
}));
if (result.resetLabel !== '110%' || result.scale !== '1.1' ||
    result.stored !== '110' || result.toolbarWidth !== 34) {
  throw new Error(`zoom in failed: ${JSON.stringify(result)}`);
}

const modifier = process.platform === 'darwin' ? 'Meta' : 'Control';
await page.keyboard.press(`${modifier}+0`);
result = await page.evaluate(() => ({
  resetLabel: document.getElementById('btn-zoom-reset').textContent,
  scale: getComputedStyle(document.documentElement).getPropertyValue('--content-scale').trim(),
}));
if (result.resetLabel !== '100%' || result.scale !== '1') {
  throw new Error(`zoom reset failed: ${JSON.stringify(result)}`);
}

await page.evaluate((source) => {
  const preview = document.getElementById('preview');
  preview.innerHTML = Array.from(
    { length: 100 },
    (_, index) => `<p>Preview paragraph ${index + 1}</p>`,
  ).join('');
  document.getElementById('editor').value = source;
  const max = document.documentElement.scrollHeight - innerHeight;
  scrollTo(0, max * 0.5);
}, editorText);
const previewProgress = await page.evaluate(
  () => scrollY / (document.documentElement.scrollHeight - innerHeight),
);
await page.locator('#btn-toggle').click();
await page.waitForTimeout(100);
const editorState = await page.evaluate(() => ({
  progress: scrollY / (document.documentElement.scrollHeight - innerHeight),
  editing: document.body.classList.contains('editing'),
  scrollY,
  scrollHeight: document.documentElement.scrollHeight,
  editorHeight: document.getElementById('editor').getBoundingClientRect().height,
  editorScrollHeight: document.getElementById('editor').scrollHeight,
}));
const editorProgress = editorState.progress;
await page.locator('#btn-toggle').click();
await page.waitForTimeout(100);
const restoredPreviewProgress = await page.evaluate(
  () => scrollY / (document.documentElement.scrollHeight - innerHeight),
);

if (Math.abs(previewProgress - editorProgress) > 0.03 ||
    Math.abs(previewProgress - restoredPreviewProgress) > 0.03) {
  throw new Error(`scroll progress drifted: ${JSON.stringify({
    previewProgress,
    editorProgress,
    restoredPreviewProgress,
    editorState,
  })}`);
}

for (const width of [390, 900, 1920]) {
  await page.setViewportSize({ width, height: 844 });
  await page.evaluate(() => document.body.classList.remove('editing'));
  const available = await page.evaluate(() => document.documentElement.clientWidth);
  const appWidth = await page.locator('#app').evaluate(el => el.getBoundingClientRect().width);
  if (Math.abs(appWidth - available) > 2) throw new Error(`Document wastes window width: ${appWidth}/${available}`);
}
const diagramCss = mainRs.split('\n').filter(line => line.startsWith('#preview .mdp-mermaid ')).join('\n')
  .replaceAll('{{', '{').replaceAll('}}', '}');
await page.addStyleTag({ content: diagramCss });
await page.addScriptTag({ path: resolve(root, 'assets/mermaid/mermaid.min.js') });
await page.evaluate(async () => {
  mermaid.initialize({ startOnLoad: false, securityLevel: 'strict' });
  const { svg } = await mermaid.render('zoom-diagram', 'graph LR; A[Readable diagram] --> B[Zoomed node]');
  document.getElementById('preview').innerHTML = '<div class="mdp-mermaid">' + svg + '</div>';
});
await page.keyboard.press(`${modifier}+0`);
const nodeWidth = await page.locator('.mdp-mermaid .node').first().evaluate(el => el.getBoundingClientRect().width);
for (let i = 0; i < 10; i++) await page.keyboard.press(`${modifier}+=`);
const zoomedNodeWidth = await page.locator('.mdp-mermaid .node').first().evaluate(el => el.getBoundingClientRect().width);
if (Math.abs(zoomedNodeWidth / nodeWidth - 2) > 0.05) {
  throw new Error(`Mermaid did not zoom with content: ${nodeWidth} -> ${zoomedNodeWidth}`);
}
await page.setViewportSize({ width: 390, height: 844 });
const diagramOverflow = await page.evaluate(() => ({
  document: document.documentElement.scrollWidth - document.documentElement.clientWidth,
  diagram: document.querySelector('.mdp-mermaid').scrollWidth - document.querySelector('.mdp-mermaid').clientWidth,
}));
if (diagramOverflow.diagram <= 1) throw new Error(`Enlarged diagram must scroll locally: ${JSON.stringify(diagramOverflow)}`);
if (diagramOverflow.document > 1) throw new Error(`Diagram zoom overflows the document: ${JSON.stringify(diagramOverflow)}`);
await page.keyboard.press(`${modifier}+0`);
const narrowBase = await page.locator('.mdp-mermaid .node').first().evaluate(el => el.getBoundingClientRect().width);
for (let i = 0; i < 10; i++) await page.keyboard.press(`${modifier}+=`);
const narrowZoom = await page.locator('.mdp-mermaid .node').first().evaluate(el => el.getBoundingClientRect().width);
if (Math.abs(narrowZoom / narrowBase - 2) > 0.05) throw new Error(`Constrained diagram does not zoom: ${narrowBase} -> ${narrowZoom}`);
await browser.close();
console.log('[desktop-reading-tools-verify] OK');
