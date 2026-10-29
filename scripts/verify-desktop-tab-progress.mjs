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
      .tabbar {
        position: sticky;
        top: 0;
        z-index: 110;
        height: 40px;
        display: flex;
        align-items: stretch;
      }
      .tabs { display: flex; flex: 1; min-width: 0; overflow-x: auto; }
      .tab { display: flex; align-items: center; }
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
      <div id="preview"></div>
      <textarea id="editor"></textarea>
    </div>
    <script>
      window.__messages = [];
      window.ipc = { postMessage(message) { window.__messages.push(message); } };
    </script>
    <script>${desktopScript}</script>
  </body>
</html>`);

await page.evaluate(() => {
  window.__testTabs = (activeId, ids) => window.__setTabs(ids.map((id) => ({
    id,
    name: `doc-${id}.md`,
    path: `/tmp/doc-${id}.md`,
    active: id === activeId,
    missing: false,
    dirty: false,
  })));
  window.__testDoc = (label) => Array.from(
    { length: 120 },
    (_, index) => `<p>${label} paragraph ${index + 1}</p>`,
  ).join('');
  window.__settle = () => new Promise((done) => requestAnimationFrame(() => requestAnimationFrame(done)));
});

const showTab = async (activeId, ids, tabId, label) => {
  await page.evaluate(({ activeId, ids, tabId, label }) => {
    window.__setContent(window.__testDoc(label), label, '', false, false, tabId);
    window.__testTabs(activeId, ids);
  }, { activeId, ids, tabId, label });
  await page.evaluate(() => window.__settle());
};

const scrollTo = async (top) => {
  await page.evaluate((value) => window.scrollTo(0, value), top);
  await page.evaluate(() => window.__settle());
};

const scrollY = () => page.evaluate(() => window.scrollY);

await page.evaluate(() => window.__testTabs(1, [1, 2]));
await showTab(1, [1, 2], 1, 'Doc one');
if (await scrollY() !== 0) throw new Error('a fresh tab must start at the top');

await scrollTo(800);
if (Math.abs(await scrollY() - 800) > 6) throw new Error(`failed to scroll the first tab: ${await scrollY()}`);

await page.evaluate(() => document.querySelector('.tab[data-tab-id="2"]').click());
const activated = await page.evaluate(() => window.__messages.includes('tab-action:activate:2'));
if (!activated) throw new Error('clicking a tab must request activation through Rust');

await showTab(2, [1, 2], 2, 'Doc two');
if (await scrollY() !== 0) {
  throw new Error(`a tab without a remembered position must open at the top, got ${await scrollY()}`);
}

await scrollTo(300);
await page.evaluate(() => document.querySelector('.tab[data-tab-id="1"]').click());
await showTab(1, [1, 2], 1, 'Doc one');
if (Math.abs(await scrollY() - 800) > 6) {
  throw new Error(`switching back must restore tab one's position, got ${await scrollY()}`);
}

await page.evaluate(() => document.querySelector('.tab[data-tab-id="2"]').click());
await showTab(2, [1, 2], 2, 'Doc two');
if (Math.abs(await scrollY() - 300) > 6) {
  throw new Error(`switching back must restore tab two's position, got ${await scrollY()}`);
}

await showTab(3, [1, 2, 3], 3, 'Doc three');
if (await scrollY() !== 0) {
  throw new Error(`a newly opened tab must not inherit the previous tab's offset, got ${await scrollY()}`);
}

await page.evaluate(() => document.querySelector('.tab[data-tab-id="1"]').click());
await showTab(1, [1, 2, 3], 1, 'Doc one');
if (Math.abs(await scrollY() - 800) > 6) {
  throw new Error(`tab one's position must survive opening other tabs, got ${await scrollY()}`);
}

await showTab(1, [1, 2, 3], 1, 'Doc one reloaded');
if (Math.abs(await scrollY() - 800) > 6) {
  throw new Error(`reloading the active tab must keep its position, got ${await scrollY()}`);
}

await browser.close();
console.log('[tab-progress] desktop tab scroll positions are remembered per tab');