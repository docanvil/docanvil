// Tab switching
document.querySelectorAll('.tab-header').forEach(btn => {
  btn.addEventListener('click', () => {
    const container = btn.closest('.tabs, .code-group');
    const tab = btn.dataset.tab;
    container.querySelectorAll('.tab-header').forEach(b => b.classList.remove('active'));
    container.querySelectorAll('.tab-content').forEach(c => c.classList.remove('active'));
    btn.classList.add('active');
    container.querySelector(`.tab-content[data-tab="${tab}"]`).classList.add('active');
  });
});

// Sidebar collapse/expand
document.querySelectorAll('.nav-group-toggle').forEach(btn => {
  btn.addEventListener('click', () => {
    const group = btn.closest('.nav-group');
    const expanded = group.classList.toggle('open');
    btn.setAttribute('aria-expanded', expanded);
  });
});

// Clickable heading anchor links
document.querySelectorAll('.content h1[id], .content h2[id], .content h3[id], .content h4[id], .content h5[id], .content h6[id]').forEach(heading => {
  const link = document.createElement('a');
  link.className = 'heading-anchor';
  link.href = '#' + heading.id;
  link.setAttribute('aria-label', 'Link to this heading');
  link.innerHTML = '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/></svg>';
  heading.appendChild(link);
});

// Copy button on code blocks
document.querySelectorAll('.content pre:not(.mermaid)').forEach(pre => {
  const btn = document.createElement('button');
  btn.className = 'copy-btn';
  btn.setAttribute('aria-label', 'Copy code');
  btn.innerHTML = '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>';
  pre.appendChild(btn);

  btn.addEventListener('click', () => {
    const code = pre.querySelector('code');
    const text = (code || pre).textContent;
    navigator.clipboard.writeText(text).then(() => {
      btn.classList.add('copied');
      btn.innerHTML = '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>';
      setTimeout(() => {
        btn.classList.remove('copied');
        btn.innerHTML = '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>';
      }, 2000);
    });
  });
});

// Popover edge-avoidance
document.querySelectorAll('.popover-trigger').forEach(trigger => {
  const popover = trigger.querySelector('.popover-content');
  if (!popover) return;

  function reposition() {
    // Reset positioning classes
    popover.classList.remove('popover-flipped');
    popover.style.left = '';
    popover.style.transform = '';

    const triggerRect = trigger.getBoundingClientRect();
    const popoverRect = popover.getBoundingClientRect();

    // Flip below if overflowing top
    if (popoverRect.top < 0) {
      popover.classList.add('popover-flipped');
    }

    // Shift horizontally if overflowing edges
    const updated = popover.getBoundingClientRect();
    if (updated.left < 8) {
      const shift = 8 - updated.left;
      popover.style.left = `calc(50% + ${shift}px)`;
    } else if (updated.right > window.innerWidth - 8) {
      const shift = updated.right - (window.innerWidth - 8);
      popover.style.left = `calc(50% - ${shift}px)`;
    }
  }

  trigger.addEventListener('mouseenter', reposition);
  trigger.addEventListener('focusin', reposition);
});

// Mobile navigation toggle
(function() {
  const toggle = document.querySelector('.mobile-nav-toggle');
  const sidebar = document.querySelector('.sidebar');
  if (!toggle || !sidebar) return;

  toggle.addEventListener('click', () => {
    const isOpen = sidebar.classList.toggle('mobile-open');
    document.body.classList.toggle('nav-open', isOpen);
    toggle.setAttribute('aria-expanded', isOpen);
  });

  document.addEventListener('click', (e) => {
    if (sidebar.classList.contains('mobile-open') &&
        !sidebar.contains(e.target) &&
        !toggle.contains(e.target)) {
      sidebar.classList.remove('mobile-open');
      document.body.classList.remove('nav-open');
      toggle.setAttribute('aria-expanded', 'false');
    }
  });
})();

// Table of contents
(function() {
  const tocNav = document.querySelector('.toc-nav');
  if (!tocNav) return;
  const multiH1 = document.querySelectorAll('.content h1[id]').length > 1;
  const headings = document.querySelectorAll(
    multiH1
      ? '.content h1[id], .content h2[id], .content h3[id]'
      : '.content h2[id], .content h3[id]'
  );
  if (headings.length < 2) {
    tocNav.closest('.toc').style.display = 'none';
    return;
  }

  const ul = document.createElement('ul');
  ul.className = 'toc-list' + (multiH1 ? ' toc-has-h1' : '');
  headings.forEach(h => {
    const li = document.createElement('li');
    let cls = 'toc-item';
    if (h.tagName === 'H3') cls += ' toc-h3';
    else if (h.tagName === 'H2' && multiH1) cls += ' toc-h2';
    li.className = cls;
    const a = document.createElement('a');
    a.href = '#' + h.id;
    a.textContent = h.textContent.replace(/\s*#$/, '');
    li.appendChild(a);
    ul.appendChild(li);
  });
  tocNav.appendChild(ul);

  const observer = new IntersectionObserver(entries => {
    entries.forEach(entry => {
      if (entry.isIntersecting) {
        tocNav.querySelectorAll('.toc-item').forEach(item => item.classList.remove('active'));
        const active = tocNav.querySelector(`a[href="#${entry.target.id}"]`);
        if (active) active.parentElement.classList.add('active');
      }
    });
  }, { rootMargin: '-80px 0px -70% 0px' });

  headings.forEach(h => observer.observe(h));
})();

// Theme toggle (light/dark)
(function() {
  var toggle = document.querySelector('.theme-toggle');
  if (!toggle) return;

  var iconLight = toggle.querySelector('.theme-icon-light');
  var iconDark = toggle.querySelector('.theme-icon-dark');

  function getEffectiveTheme() {
    var stored = localStorage.getItem('docanvil-theme');
    if (stored === 'dark' || stored === 'light') return stored;
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
  }

  function applyTheme(theme) {
    document.documentElement.setAttribute('data-theme', theme);
    if (iconLight && iconDark) {
      iconLight.style.display = theme === 'dark' ? 'none' : 'block';
      iconDark.style.display = theme === 'dark' ? 'block' : 'none';
    }
  }

  applyTheme(getEffectiveTheme());

  toggle.addEventListener('click', function() {
    var current = getEffectiveTheme();
    var next = current === 'dark' ? 'light' : 'dark';
    localStorage.setItem('docanvil-theme', next);
    applyTheme(next);
  });

  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', function() {
    if (!localStorage.getItem('docanvil-theme')) {
      applyTheme(getEffectiveTheme());
    }
  });
})();

// Locale switcher
(function() {
  var switcher = document.querySelector('.locale-switcher');
  if (!switcher) return;

  var trigger = switcher.querySelector('.locale-switcher-trigger');
  var menu = switcher.querySelector('.locale-switcher-menu');

  trigger.addEventListener('click', function(e) {
    e.stopPropagation();
    var isOpen = switcher.classList.toggle('open');
    trigger.setAttribute('aria-expanded', isOpen);
  });

  document.addEventListener('click', function(e) {
    if (!switcher.contains(e.target)) {
      switcher.classList.remove('open');
      trigger.setAttribute('aria-expanded', 'false');
    }
  });

  // Store locale choice on explicit click
  menu.addEventListener('click', function(e) {
    var link = e.target.closest('a[data-locale]');
    if (link) {
      localStorage.setItem('docanvil-locale', link.dataset.locale);
    }
  });
})();

// Locale auto-detection
(function() {
  var html = document.documentElement;
  var autoDetect = document.body.dataset.localeAutoDetect === 'true';
  if (!autoDetect) return;

  var currentLocale = html.getAttribute('lang') || 'en';
  var switcher = document.querySelector('.locale-switcher-menu');
  if (!switcher) return;

  // Collect available locale codes
  var links = switcher.querySelectorAll('a[data-locale]');
  var available = [];
  var urlMap = {};
  for (var i = 0; i < links.length; i++) {
    var code = links[i].dataset.locale;
    available.push(code);
    urlMap[code] = links[i].href;
  }

  // Check localStorage first
  var saved = localStorage.getItem('docanvil-locale');
  if (saved) return; // User already made an explicit choice

  // Check navigator.language
  var browserLang = (navigator.language || '').split('-')[0].toLowerCase();
  if (browserLang && browserLang !== currentLocale && available.indexOf(browserLang) >= 0) {
    localStorage.setItem('docanvil-locale', browserLang);
    window.location.href = urlMap[browserLang];
  }
})();

// Search
(function() {
  var overlay = document.querySelector('.search-overlay');
  var backdrop = document.querySelector('.search-overlay-backdrop');
  var input = document.querySelector('.search-overlay-input');
  var resultsContainer = document.querySelector('.search-overlay-results');
  var trigger = document.querySelector('.search-trigger');
  if (!overlay || !input || !resultsContainer) return;

  var baseUrl = document.body.dataset.baseUrl || '/';
  var miniSearch = null;
  var loaded = false;
  var selectedIndex = -1;

  function getSnippet(body, query, maxLen) {
    if (!body) return '';
    var lower = body.toLowerCase();
    var terms = query.toLowerCase().split(/\s+/).filter(Boolean);
    var pos = -1;
    for (var t = 0; t < terms.length; t++) {
      pos = lower.indexOf(terms[t]);
      if (pos >= 0) break;
    }
    if (pos < 0) return body.slice(0, maxLen) + (body.length > maxLen ? '...' : '');
    var start = Math.max(0, pos - 60);
    var end = Math.min(body.length, pos + maxLen - 60);
    var snippet = (start > 0 ? '...' : '') + body.slice(start, end) + (end < body.length ? '...' : '');
    for (var t = 0; t < terms.length; t++) {
      var re = new RegExp('(' + terms[t].replace(/[.*+?^${}()|[\]\\]/g, '\\$&') + ')', 'gi');
      snippet = snippet.replace(re, '<mark>$1</mark>');
    }
    return snippet;
  }

  function escapeHtml(s) {
    return s.replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;').replace(/"/g,'&quot;');
  }

  function loadSearch() {
    if (loaded) return;
    loaded = true;
    var script = document.createElement('script');
    script.src = 'https://cdn.jsdelivr.net/npm/minisearch@7/dist/umd/index.min.js';
    script.onload = function() {
      fetch(baseUrl + 'search-index.json')
        .then(function(r) { return r.json(); })
        .then(function(data) {
          miniSearch = new MiniSearch({
            fields: ['title', 'heading', 'body'],
            storeFields: ['title', 'heading', 'url', 'body', 'breadcrumbs'],
            searchOptions: {
              boost: { title: 3, heading: 2 },
              prefix: true,
              fuzzy: 0.2
            }
          });
          miniSearch.addAll(data);
          if (input.value) doSearch(input.value);
        });
    };
    document.head.appendChild(script);
  }

  function openOverlay() {
    loadSearch();
    overlay.classList.add('open');
    document.body.style.overflow = 'hidden';
    input.focus();
  }

  function closeOverlay() {
    overlay.classList.remove('open');
    document.body.style.overflow = '';
    input.value = '';
    resultsContainer.innerHTML = '';
    selectedIndex = -1;
  }

  function doSearch(query) {
    selectedIndex = -1;
    if (!miniSearch || !query.trim()) {
      resultsContainer.innerHTML = query.trim() && !miniSearch ? '<div class="search-no-results">Loading...</div>' : '';
      return;
    }
    var results = miniSearch.search(query).slice(0, 10);
    if (results.length === 0) {
      resultsContainer.innerHTML = '<div class="search-no-results">No results found</div>';
      return;
    }
    resultsContainer.innerHTML = results.map(function(r, i) {
      var trail = r.breadcrumbs && r.breadcrumbs.length ? r.breadcrumbs.slice() : [r.title];
      if (r.heading) trail.push(r.heading);
      var titleText = '';
      for (var t = 0; t < trail.length; t++) {
        if (t > 0) titleText += '<span class="search-result-separator search-result-breadcrumb">&rsaquo;</span>';
        if (t < trail.length - 1) {
          titleText += '<span class="search-result-breadcrumb">' + escapeHtml(trail[t]) + '</span>';
        } else {
          titleText += escapeHtml(trail[t]);
        }
      }
      var snippet = getSnippet(r.body || '', query, 140);
      return '<a class="search-result-item" href="' + r.url + '" role="option" data-index="' + i + '">' +
        '<div class="search-result-title">' + titleText + '</div>' +
        (snippet ? '<div class="search-result-snippet">' + snippet + '</div>' : '') +
        '</a>';
    }).join('');
  }

  function updateSelection() {
    var items = resultsContainer.querySelectorAll('.search-result-item');
    items.forEach(function(item, i) {
      item.classList.toggle('selected', i === selectedIndex);
    });
    if (selectedIndex >= 0 && items[selectedIndex]) {
      items[selectedIndex].scrollIntoView({ block: 'nearest' });
    }
  }

  if (trigger) {
    trigger.addEventListener('click', openOverlay);
  }

  backdrop.addEventListener('click', closeOverlay);

  input.addEventListener('input', function() { doSearch(input.value); });

  input.addEventListener('keydown', function(e) {
    var items = resultsContainer.querySelectorAll('.search-result-item');
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      selectedIndex = Math.min(selectedIndex + 1, items.length - 1);
      updateSelection();
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      selectedIndex = Math.max(selectedIndex - 1, -1);
      updateSelection();
    } else if (e.key === 'Enter' && selectedIndex >= 0 && items[selectedIndex]) {
      e.preventDefault();
      items[selectedIndex].click();
    } else if (e.key === 'Escape') {
      e.preventDefault();
      closeOverlay();
    }
  });

  resultsContainer.addEventListener('click', function(e) {
    if (e.target.closest('.search-result-item')) {
      closeOverlay();
    }
  });

  document.addEventListener('keydown', function(e) {
    if (e.key === '/' && !overlay.classList.contains('open')) {
      var tag = (e.target.tagName || '').toLowerCase();
      if (tag === 'input' || tag === 'textarea' || tag === 'select' || e.target.isContentEditable) return;
      e.preventDefault();
      openOverlay();
    }
    if (e.key === 'Escape' && overlay.classList.contains('open')) {
      closeOverlay();
    }
  });
})();
