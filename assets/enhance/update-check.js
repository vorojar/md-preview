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

  function isDesktopReleaseTag(tagName) {
    return typeof tagName === 'string' && /^v\d+(?:\.\d+)+$/.test(tagName.trim());
  }

  function selectDesktopRelease(payload, currentVersion) {
    var releases = Array.isArray(payload) ? payload : [payload];
    var best = null;

    for (var i = 0; i < releases.length; i++) {
      var release = releases[i];
      if (!release || release.draft || release.prerelease) continue;
      if (!isDesktopReleaseTag(release.tag_name)) continue;
      if (!isNewerVersion(release.tag_name, currentVersion)) continue;
      if (!best || isNewerVersion(release.tag_name, best.tag_name)) {
        best = release;
      }
    }

    return best;
  }

  function preferredAssetPattern() {
    var nav = typeof navigator !== 'undefined' ? navigator : {};
    var platform = (nav.platform || '').toLowerCase();
    var ua = (nav.userAgent || '').toLowerCase();

    if (platform.indexOf('mac') >= 0 || ua.indexOf('mac os') >= 0) {
      return /macos.*\.dmg$/i;
    }
    if (platform.indexOf('win') >= 0 || ua.indexOf('windows') >= 0) {
      return /^MD-Preview-windows-x64\.exe$/i;
    }
    if (platform.indexOf('linux') >= 0 || ua.indexOf('linux') >= 0) {
      return /linux.*\.tar\.gz$/i;
    }
    return null;
  }

  function selectDownloadAsset(release) {
    var pattern = preferredAssetPattern();
    var assets = release && Array.isArray(release.assets) ? release.assets : [];
    if (!pattern) return null;

    for (var i = 0; i < assets.length; i++) {
      var asset = assets[i];
      if (asset && pattern.test(asset.name || '') && asset.browser_download_url) {
        return asset;
      }
    }

    return null;
  }

  function selectDownloadUrl(release) {
    var asset = selectDownloadAsset(release);
    if (asset) return asset.browser_download_url;
    return release && release.html_url;
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

    function postUpdateResult(kind, release) {
      if (!window.ipc) return;
      if (kind === 'available') {
        var asset = selectDownloadAsset(release);
        window.ipc.postMessage([
          'update-check-result:available',
          release && release.tag_name || '',
          release && selectDownloadUrl(release) || latestUrl,
          asset && asset.digest || release && release.download_digest || ''
        ].join('\n'));
      }
      else window.ipc.postMessage('update-check-result:' + kind);
    }

    function openRelease() {
      var url = button.dataset.releaseUrl || latestUrl;
      if (config.nativeUpdater && window.ipc) {
        window.ipc.postMessage([
          'check-updates:',
          url,
          button.dataset.releaseDigest || '',
          button.dataset.releaseTag || ''
        ].join('\n'));
      }
      else if (window.ipc) window.ipc.postMessage('open-url:' + url);
      else window.location.href = url;
    }

    function applyRelease(release) {
      if (!release || !release.tag_name) return false;
      if (!isNewerVersion(release.tag_name, config.currentVersion)) return false;

      var asset = selectDownloadAsset(release);
      var url = release.download_url || (asset && asset.browser_download_url) || release.html_url || latestUrl;
      button.dataset.releaseUrl = url;
      button.dataset.releaseDigest = (asset && asset.digest) || release.download_digest || '';
      button.dataset.releaseTag = release.tag_name || '';
      button.title = label + ': ' + release.tag_name;
      button.setAttribute('aria-label', button.title);
      var text = button.querySelector ? button.querySelector('.update-label') : null;
      if (text) text.textContent = label;
      button.hidden = false;
      if (button.parentElement) button.parentElement.classList.add('has-update');
      return true;
    }

    button.addEventListener('click', openRelease);

    window.__mdPreviewApplyUpdateRelease = applyRelease;
    window.__mdPreviewCheckUpdates = function(showDialog) {
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
        .then(function(payload) {
          var release = selectDesktopRelease(payload, config.currentVersion);
          if (release) {
            release.download_digest = (function() {
              var asset = selectDownloadAsset(release);
              return asset && asset.digest;
            })();
          }
          if (!showDialog) {
            storageSet(storageKey, JSON.stringify({
              checkedAt: Date.now(),
              currentVersion: config.currentVersion,
              tagName: release && release.tag_name,
              htmlUrl: release && release.html_url,
              downloadUrl: release && selectDownloadUrl(release),
              downloadDigest: release && release.download_digest
            }));
          }
          applyRelease(release);
          if (showDialog) postUpdateResult(release ? 'available' : 'none', release);
        })
        .catch(function() {
          if (showDialog) postUpdateResult('failed');
        })
        .then(function() {
          if (timer) clearTimeout(timer);
        });
    };

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
            applyRelease({
              tag_name: parsed.tagName,
              html_url: parsed.htmlUrl,
              download_url: parsed.downloadUrl,
              download_digest: parsed.downloadDigest
            });
            return;
          }
        } catch (e) {}
      }

      window.__mdPreviewCheckUpdates(false);
    });
  }

  window.__mdPreviewInstallUpdateCheck = installUpdateCheck;
  window.__mdPreviewIsNewerVersion = isNewerVersion;
  window.__mdPreviewSelectDesktopRelease = selectDesktopRelease;
  window.__mdPreviewSelectDownloadUrl = selectDownloadUrl;
})();
