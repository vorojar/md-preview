(function() {
  var flags = window.__mdPreviewFeatureFlags || { math: false, mermaid: false };
  var mermaidSeq = 0;
  var anchorNavigationBound = false;

  function idle(fn) {
    return (window.requestIdleCallback || function(cb) { return setTimeout(cb, 0); })(fn);
  }

  function setFlags(needsMath, needsMermaid) {
    flags = { math: !!needsMath, mermaid: !!needsMermaid };
    window.__mdPreviewFeatureFlags = flags;
  }

  function hasClassInTree(el, cls) {
    while (el && el !== document.body) {
      if (el.classList && el.classList.contains(cls)) return true;
      el = el.parentElement;
    }
    return false;
  }

  function shouldSkipTextNode(node) {
    var el = node.parentElement;
    if (!el) return true;
    var tag = el.tagName;
    if (tag === 'CODE' || tag === 'PRE' || tag === 'KBD' || tag === 'SAMP' ||
        tag === 'SCRIPT' || tag === 'STYLE' || tag === 'TEXTAREA') return true;
    return hasClassInTree(el, 'katex') ||
      hasClassInTree(el, 'math') ||
      hasClassInTree(el, 'mdp-mark') ||
      hasClassInTree(el, 'mdp-mermaid') ||
      hasClassInTree(el, 'mdp-mermaid-error');
  }

  function isEscaped(text, index) {
    var count = 0;
    for (var i = index - 1; i >= 0 && text[i] === '\\'; i--) count++;
    return count % 2 === 1;
  }

  function findClosing(text, start, delim) {
    for (var i = start; i < text.length; i++) {
      if (delim === '$') {
        if (text[i] === '$' && !isEscaped(text, i) && text[i - 1] && !/\s/.test(text[i - 1])) {
          return i;
        }
      } else if (text.substr(i, delim.length) === delim && !isEscaped(text, i)) {
        return i;
      }
    }
    return -1;
  }

  function nextDelimiter(text, index) {
    if (text.substr(index, 2) === '$$' && !isEscaped(text, index)) {
      return { open: '$$', close: '$$', display: true };
    }
    if (text.substr(index, 2) === '\\[' && !isEscaped(text, index)) {
      return { open: '\\[', close: '\\]', display: true };
    }
    if (text.substr(index, 2) === '\\(' && !isEscaped(text, index)) {
      return { open: '\\(', close: '\\)', display: false };
    }
    if (text[index] === '$' && !isEscaped(text, index) && text[index + 1] &&
        text[index + 1] !== '$' && !/\s/.test(text[index + 1])) {
      return { open: '$', close: '$', display: false };
    }
    return null;
  }

  function mathParts(text) {
    var parts = [];
    var i = 0;
    var last = 0;
    while (i < text.length) {
      var delim = nextDelimiter(text, i);
      if (!delim) {
        i++;
        continue;
      }
      var bodyStart = i + delim.open.length;
      var closeAt = findClosing(text, bodyStart, delim.close);
      if (closeAt < 0) {
        i += delim.open.length;
        continue;
      }
      if (i > last) parts.push({ text: text.slice(last, i) });
      parts.push({
        math: text.slice(bodyStart, closeAt),
        display: delim.display
      });
      i = closeAt + delim.close.length;
      last = i;
    }
    if (!parts.length) return null;
    if (last < text.length) parts.push({ text: text.slice(last) });
    return parts;
  }

  function enhanceMath(root) {
    if (!flags.math || !window.katex) return;
    var mathNodes = root.querySelectorAll('.math.math-inline, .math.math-display');
    Array.prototype.forEach.call(mathNodes, function(el) {
      if (el.dataset.mdpMath === '1') return;
      var source = el.textContent;
      var display = el.classList.contains('math-display');
      el.dataset.mdpMath = '1';
      try {
        window.katex.render(source, el, {
          displayMode: display,
          throwOnError: false,
          strict: 'warn',
          trust: false
        });
      } catch (err) {
        el.className = 'mdp-math-error';
        el.textContent = source;
        el.title = err && err.message ? err.message : 'KaTeX render error';
      }
    });

    var walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT, {
      acceptNode: function(node) {
        if (shouldSkipTextNode(node)) return NodeFilter.FILTER_REJECT;
        return /(\$|\\\(|\\\[)/.test(node.nodeValue) ?
          NodeFilter.FILTER_ACCEPT :
          NodeFilter.FILTER_REJECT;
      }
    });
    var nodes = [];
    while (walker.nextNode()) nodes.push(walker.currentNode);
    nodes.forEach(function(node) {
      var parts = mathParts(node.nodeValue);
      if (!parts) return;
      var frag = document.createDocumentFragment();
      parts.forEach(function(part) {
        if (part.text !== undefined) {
          frag.appendChild(document.createTextNode(part.text));
          return;
        }
        var span = document.createElement('span');
        try {
          window.katex.render(part.math, span, {
            displayMode: part.display,
            throwOnError: false,
            strict: 'warn',
            trust: false
          });
        } catch (err) {
          span.className = 'mdp-math-error';
          span.textContent = part.math;
          span.title = err && err.message ? err.message : 'KaTeX render error';
        }
        frag.appendChild(span);
      });
      node.parentNode.replaceChild(frag, node);
    });
  }

  function currentMermaidTheme() {
    return window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches ?
      'dark' :
      'default';
  }

  function renderMermaid(container, source) {
    if (!window.mermaid || !window.mermaid.render) return;
    var id = 'mdp-mermaid-' + (++mermaidSeq);
    try {
      window.mermaid.initialize({
        startOnLoad: false,
        securityLevel: 'strict',
        theme: currentMermaidTheme()
      });
      Promise.resolve(window.mermaid.render(id, source)).then(function(result) {
        container.innerHTML = result.svg || result;
      }).catch(function(err) {
        container.className = 'mdp-mermaid-error';
        container.textContent = source;
        container.title = err && err.message ? err.message : 'Mermaid render error';
      });
    } catch (err) {
      container.className = 'mdp-mermaid-error';
      container.textContent = source;
      container.title = err && err.message ? err.message : 'Mermaid render error';
    }
  }

  function enhanceMermaid(root) {
    if (!flags.mermaid || !window.mermaid) return;
    var nodes = root.querySelectorAll('pre > code.language-mermaid, pre > code.mermaid');
    Array.prototype.forEach.call(nodes, function(code) {
      var pre = code.parentElement;
      if (!pre || pre.dataset.mdpMermaid === '1') return;
      pre.dataset.mdpMermaid = '1';
      var source = code.textContent;
      var container = document.createElement('div');
      container.className = 'mdp-mermaid';
      container.textContent = source;
      pre.parentNode.replaceChild(container, pre);
      renderMermaid(container, source);
    });
  }

  function enhanceTables(root) {
    var tables = root.querySelectorAll('table');
    Array.prototype.forEach.call(tables, function(table) {
      if (hasClassInTree(table, 'mdp-table-wrap')) return;
      var firstRow = table.querySelector('tr');
      var cellCount = firstRow ? firstRow.children.length : 0;
      if (cellCount < 4) return;

      var wrap = document.createElement('div');
      wrap.className = 'mdp-table-wrap';
      table.parentNode.insertBefore(wrap, table);
      wrap.appendChild(table);
    });
  }

  function alertClassFor(value) {
    var type = String(value || '').toLowerCase();
    if (type === 'note' || type === 'tip' || type === 'important' ||
        type === 'warning' || type === 'caution') {
      return 'markdown-alert-' + type;
    }
    return '';
  }

  function alertTypeFromClass(node) {
    var match = String(node && node.className || '').match(/\bmarkdown-alert-(note|tip|important|warning|caution)\b/i);
    return match ? match[1].toLowerCase() : '';
  }

  function alertLabelFor(type) {
    var labels = {
      note: 'Note',
      tip: 'Tip',
      important: 'Important',
      warning: 'Warning',
      caution: 'Caution'
    };
    return labels[type] || '';
  }

  function ensureAlertTitle(quote, type) {
    if (!quote || !type) return;
    var existing = quote.firstElementChild;
    if (existing && existing.classList && existing.classList.contains('markdown-alert-title')) return;

    var title = document.createElement('p');
    title.className = 'markdown-alert-title';
    title.textContent = alertLabelFor(type);
    quote.insertBefore(title, quote.firstChild);
  }

  function trimAlertLead(node) {
    while (node && node.firstChild) {
      var child = node.firstChild;
      if (child.nodeType === Node.TEXT_NODE && !child.nodeValue.trim()) {
        node.removeChild(child);
        continue;
      }
      if (child.nodeName === 'BR') {
        node.removeChild(child);
        continue;
      }
      break;
    }
  }

  function enhanceAlerts(root) {
    var quotes = root.querySelectorAll('blockquote');
    Array.prototype.forEach.call(quotes, function(quote) {
      var type = alertTypeFromClass(quote);
      var first = quote.firstElementChild || quote;
      var match = null;

      if (!type && first && first.firstChild && first.firstChild.nodeType === Node.TEXT_NODE) {
        match = first.firstChild.nodeValue.match(/^\s*\[!(NOTE|TIP|IMPORTANT|WARNING|CAUTION)\]\s*/i);
        if (match) type = String(match[1]).toLowerCase();
      }

      if (!type) return;

      var cls = alertClassFor(type);
      if (!cls) return;

      quote.classList.add(cls);
      if (match) {
        first.firstChild.nodeValue = first.firstChild.nodeValue.slice(match[0].length);
        trimAlertLead(first);
        if (first !== quote && !first.children.length && !first.textContent.trim()) {
          quote.removeChild(first);
        }
        trimAlertLead(quote);
      }
      ensureAlertTitle(quote, type);
    });
  }

  function markParts(text) {
    var parts = [];
    var cursor = 0;
    while (cursor < text.length) {
      var open = text.indexOf('==', cursor);
      if (open < 0) break;
      var close = text.indexOf('==', open + 2);
      if (close < 0) break;
      var body = text.slice(open + 2, close);
      if (!body.trim()) break;
      if (open > cursor) parts.push({ text: text.slice(cursor, open) });
      parts.push({ mark: body });
      cursor = close + 2;
    }
    if (!parts.length) return null;
    if (cursor < text.length) parts.push({ text: text.slice(cursor) });
    return parts;
  }

  function enhanceMarks(root) {
    var walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT, {
      acceptNode: function(node) {
        if (shouldSkipTextNode(node)) return NodeFilter.FILTER_REJECT;
        return node.nodeValue.indexOf('==') >= 0 ?
          NodeFilter.FILTER_ACCEPT :
          NodeFilter.FILTER_REJECT;
      }
    });
    var nodes = [];
    while (walker.nextNode()) nodes.push(walker.currentNode);
    nodes.forEach(function(node) {
      var parts = markParts(node.nodeValue);
      if (!parts) return;
      var frag = document.createDocumentFragment();
      parts.forEach(function(part) {
        if (part.text !== undefined) {
          frag.appendChild(document.createTextNode(part.text));
          return;
        }
        var mark = document.createElement('mark');
        mark.className = 'mdp-mark';
        mark.textContent = part.mark;
        frag.appendChild(mark);
      });
      node.parentNode.replaceChild(frag, node);
    });
  }

  function decodeFragment(value) {
    try {
      return decodeURIComponent(value);
    } catch (_) {
      return value;
    }
  }

  function updateHash(fragment) {
    try {
      if (window.history && window.history.pushState) {
        window.history.pushState(null, '', '#' + fragment);
        return;
      }
    } catch (_) {}

    try {
      window.location.hash = fragment;
    } catch (_) {}
  }

  function bindAnchorNavigation() {
    if (anchorNavigationBound) return;
    anchorNavigationBound = true;

    document.addEventListener('click', function(event) {
      var link = event.target && event.target.closest ?
        event.target.closest('#preview a[href]') :
        null;
      if (!link) return;

      var href = link.getAttribute('href') || '';
      if (href.charAt(0) !== '#') {
        var local = /^file:/i.test(href) || !/^[a-z][a-z0-9+.-]*:/i.test(href);
        if (local && window.ipc && window.ipc.postMessage) {
          event.preventDefault();
          window.ipc.postMessage('open-local-link:' + href);
        }
        return;
      }

      event.preventDefault();

      var fragment = href.slice(1);
      if (!fragment) {
        window.scrollTo(0, 0);
        updateHash(fragment);
        return;
      }

      var decoded = decodeFragment(fragment);
      var target = document.getElementById(decoded) || document.getElementById(fragment);
      if (!target) return;

      target.scrollIntoView({ block: 'start' });
      updateHash(fragment);
    });
  }

  window.__setFeatureFlags = setFlags;

  window.__setKatexCss = function(css) {
    if (document.getElementById('katex-css')) return;
    var style = document.createElement('style');
    style.id = 'katex-css';
    style.textContent = css;
    document.head.appendChild(style);
  };

  window.__enhancePreview = function() {
    var root = document.getElementById('preview');
    if (!root) return;
    bindAnchorNavigation();
    idle(function() {
      enhanceTables(root);
      enhanceAlerts(root);
      enhanceMarks(root);
      enhanceMath(root);
      enhanceMermaid(root);
    });
  };
})();
