(function() {
  'use strict';

  function parseVersion(value) {
    if (typeof value !== 'string') return null;
    var cleaned = value.trim().replace(/^v/i, '').split(/[+-]/)[0];
    if (!cleaned) return null;
    var parts = cleaned.split('.');
    var nums = [];
    for (var i = 0; i < parts.length; i++) {
      if (!/^\d+$/.test(parts[i])) return null;
      nums.push(parseInt(parts[i], 10));
    }
    return nums;
  }

  function isNewerVersion(candidate, current) {
    var next = parseVersion(candidate);
    var now = parseVersion(current);
    if (!next || !now) return false;
    var len = Math.max(next.length, now.length);
    for (var i = 0; i < len; i++) {
      var a = next[i] || 0;
      var b = now[i] || 0;
      if (a > b) return true;
      if (a < b) return false;
    }
    return false;
  }

  function storageGet(key) {
    try {
      return window.localStorage.getItem(key);
    } catch (e) {
      return null;
    }
  }

  function storageSet(key, value) {
    try {
      window.localStorage.setItem(key, value);
    } catch (e) {}
  }

  function scheduleAfterFirstPaint(fn) {
    requestAnimationFrame(function() {
      requestAnimationFrame(function() {
        var idle = window.requestIdleCallback || function(cb) {
          return setTimeout(cb, 1000);
        };
        idle(fn, { timeout: 2500 });
      });
    });
  }

  function installUpdateCheck(config) {
    if (!config || !config.currentVersion) return;

    var button = document.getElementById(config.buttonId || 'btn-update');
    if (!button) return;

    var label = config.buttonLabel || 'Update available';
    var latestUrl = config.latestUrl || 'https://github.com/vorojar/md-preview/releases/latest';
    var apiUrl = config.apiUrl || 'https://api.github.com/repos/vorojar/md-preview/releases/latest';
    var storageKey = config.storageKey || 'md-preview:update-check';
    var maxAgeMs = config.maxAgeMs || 24 * 60 * 60 * 1000;
    var timeoutMs = config.timeoutMs || 3500;

    function openRelease() {
      var url = button.dataset.releaseUrl || latestUrl;
      if (window.ipc) window.ipc.postMessage('open-url:' + url);
      else window.location.href = url;
    }

    function applyRelease(release) {
      if (!release || !release.tag_name) return false;
      if (!isNewerVersion(release.tag_name, config.currentVersion)) return false;

      var url = release.html_url || latestUrl;
      button.dataset.releaseUrl = url;
      button.title = label + ': ' + release.tag_name;
      button.setAttribute('aria-label', button.title);
      button.hidden = false;
      return true;
    }

    button.addEventListener('click', openRelease);

    window.__mdPreviewApplyUpdateRelease = applyRelease;

    scheduleAfterFirstPaint(function() {
      var now = Date.now();
      var cached = storageGet(storageKey);
      if (cached) {
        try {
          var parsed = JSON.parse(cached);
          if (
            parsed.currentVersion === config.currentVersion &&
            parsed.checkedAt &&
            now - parsed.checkedAt < maxAgeMs
          ) {
            applyRelease({ tag_name: parsed.tagName, html_url: parsed.htmlUrl });
            return;
          }
        } catch (e) {}
      }

      var controller = typeof AbortController !== 'undefined' ? new AbortController() : null;
      var timer = controller ? setTimeout(function() { controller.abort(); }, timeoutMs) : null;
      var opts = {
        cache: 'no-store',
        headers: { Accept: 'application/vnd.github+json' }
      };
      if (controller) opts.signal = controller.signal;

      fetch(apiUrl, opts)
        .then(function(response) {
          if (!response.ok) throw new Error('update check failed');
          return response.json();
        })
        .then(function(release) {
          storageSet(storageKey, JSON.stringify({
            checkedAt: now,
            currentVersion: config.currentVersion,
            tagName: release && release.tag_name,
            htmlUrl: release && release.html_url
          }));
          applyRelease(release);
        })
        .catch(function() {})
        .then(function() {
          if (timer) clearTimeout(timer);
        });
    });
  }

  window.__mdPreviewInstallUpdateCheck = installUpdateCheck;
  window.__mdPreviewIsNewerVersion = isNewerVersion;
})();
