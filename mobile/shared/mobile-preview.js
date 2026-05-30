(function() {
  var titleEl = document.getElementById('title');
  var previewEl = document.getElementById('preview');
  var baseEl = document.getElementById('base-href');
  var openButton = document.getElementById('open-file');
  var defaultSettingsButton = document.getElementById('default-settings');
  var darkSheet = document.getElementById('hljs-dark');
  var lightSheet = document.getElementById('hljs-light');
  var loaded = { katex: false, mermaid: false };

  if (window.marked && window.marked.setOptions) {
    window.marked.setOptions({
      gfm: true,
      breaks: false,
      mangle: false,
      headerIds: false,
      highlight: function(code, lang) {
        if (!window.hljs) return code;
        if (lang && window.hljs.getLanguage(lang)) {
          return window.hljs.highlight(code, { language: lang }).value;
        }
        return window.hljs.highlightAuto(code).value;
      }
    });
  }

  function applyTheme(e) {
    var dark = !!e.matches;
    lightSheet.media = dark ? 'not all' : '';
    darkSheet.media = dark ? '' : 'not all';
  }

  if (window.matchMedia) {
    var mq = window.matchMedia('(prefers-color-scheme: dark)');
    applyTheme(mq);
    if (mq.addEventListener) mq.addEventListener('change', applyTheme);
  }

  if (window.MDPreviewAndroid && window.MDPreviewAndroid.openDefaultSettings) {
    document.body.classList.add('android');
  }

  function hasUnescapedPair(text, open, close) {
    var pos = 0;
    while ((pos = text.indexOf(open, pos)) >= 0) {
      var body = pos + open.length;
      var end = text.indexOf(close, body);
      if (end > body) return true;
      pos = body;
    }
    return false;
  }

  function featureFlags(markdown) {
    return {
      math: hasUnescapedPair(markdown, '$$', '$$') ||
        hasUnescapedPair(markdown, '\\[', '\\]') ||
        hasUnescapedPair(markdown, '\\(', '\\)') ||
        /(^|[^\\])\$[^\s$][\s\S]*?[^\s\\]\$/.test(markdown),
      mermaid: /(^|\n)\s*(```|~~~)\s*mermaid(\s|\n|$)/i.test(markdown)
    };
  }

  function loadScript(src, key) {
    if (loaded[key]) return Promise.resolve();
    loaded[key] = true;
    return new Promise(function(resolve, reject) {
      var script = document.createElement('script');
      script.src = src;
      script.async = true;
      script.onload = resolve;
      script.onerror = reject;
      document.head.appendChild(script);
    });
  }

  function loadCss(href, id) {
    if (document.getElementById(id)) return;
    var link = document.createElement('link');
    link.id = id;
    link.rel = 'stylesheet';
    link.href = href;
    document.head.appendChild(link);
  }

  function idle(fn) {
    return (window.requestIdleCallback || function(cb) { return setTimeout(cb, 0); })(fn);
  }

  function enhance(flags) {
    if (window.__setFeatureFlags) window.__setFeatureFlags(flags.math, flags.mermaid);
    var tasks = [];
    if (flags.math && !window.katex) {
      loadCss('vendor/katex.inline.css', 'katex-css-link');
      tasks.push(loadScript('vendor/katex.min.js', 'katex'));
    }
    if (flags.mermaid && !window.mermaid) {
      tasks.push(loadScript('vendor/mermaid.min.js', 'mermaid'));
    }
    Promise.all(tasks).then(function() {
      if (window.__enhancePreview) window.__enhancePreview();
    });
  }

  function render(payload) {
    var markdown = payload && payload.markdown ? String(payload.markdown) : '';
    var name = payload && payload.name ? String(payload.name) : 'Untitled.md';
    var baseHref = payload && payload.baseHref ? String(payload.baseHref) : '';
    var flags = featureFlags(markdown);
    document.body.classList.remove('empty');
    titleEl.textContent = name;
    document.title = name + ' - MD Preview';
    if (baseHref) baseEl.setAttribute('href', baseHref);
    else baseEl.removeAttribute('href');
    previewEl.innerHTML = window.marked ? window.marked.parse(markdown) : markdown;
    idle(function() {
      if (window.hljs && window.hljs.highlightAll) window.hljs.highlightAll();
      enhance(flags);
    });
  }

  document.addEventListener('click', function(event) {
    var link = event.target && event.target.closest ? event.target.closest('a[href]') : null;
    if (!link) return;
    var href = link.getAttribute('href') || '';
    if (/^\s*(javascript:|data:|vbscript:)/i.test(href)) {
      event.preventDefault();
      return;
    }
    if (!/^(https?:|mailto:)/i.test(href)) return;
    event.preventDefault();
    if (window.webkit && window.webkit.messageHandlers && window.webkit.messageHandlers.mdPreview) {
      window.webkit.messageHandlers.mdPreview.postMessage({ action: 'openExternal', url: href });
      return;
    }
    if (window.MDPreviewAndroid && window.MDPreviewAndroid.openExternal) {
      window.MDPreviewAndroid.openExternal(href);
    }
  });

  openButton.addEventListener('click', function() {
    if (window.webkit && window.webkit.messageHandlers && window.webkit.messageHandlers.mdPreview) {
      window.webkit.messageHandlers.mdPreview.postMessage({ action: 'open' });
      return;
    }
    if (window.MDPreviewAndroid && window.MDPreviewAndroid.openFile) {
      window.MDPreviewAndroid.openFile();
    }
  });

  defaultSettingsButton.addEventListener('click', function() {
    if (window.MDPreviewAndroid && window.MDPreviewAndroid.openDefaultSettings) {
      window.MDPreviewAndroid.openDefaultSettings();
    }
  });

  window.MDPreview = {
    render: render,
    setEmpty: function() {
      document.body.classList.add('empty');
      titleEl.textContent = 'MD Preview';
      previewEl.innerHTML = '';
    }
  };
})();
