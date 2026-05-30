(function() {
  var flags = window.__mdPreviewFeatureFlags || { math: false, mermaid: false };
  var mermaidSeq = 0;

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
    idle(function() {
      enhanceMath(root);
      enhanceMermaid(root);
    });
  };
})();
