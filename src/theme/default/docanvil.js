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

// Bundled Devicon v2.17.0 assets (MIT); see icons/devicon/LICENSE.txt.
// Original SVGs retain their colours; monochrome logos use the mapped accent.
const codeLanguageIcons = {
  bash:       { label: 'Bash', svg: 'bash-light.svg', color: '#a8b9cc', aliases: ['sh', 'shell', 'zsh', 'console', 'shellsession', 'shell-session'] },
  rust:       { label: 'Rust', svg: 'rust.svg',       color: '#dea584', monochrome: true, aliases: ['rs'] },
  javascript: { label: 'JavaScript', svg: 'javascript.svg', color: '#f7df1e', aliases: ['js', 'jsx', 'node', 'nodejs'] },
  typescript: { label: 'TypeScript', svg: 'typescript.svg', color: '#3178c6', aliases: ['ts', 'tsx'] },
  python:     { label: 'Python', svg: 'python.svg',     color: '#3776ab', aliases: ['py', 'python3'] },
  ruby:       { label: 'Ruby', svg: 'ruby.svg',       color: '#cc342d', aliases: ['rb'] },
  go:         { label: 'Go', svg: 'go.svg',         color: '#00add8', aliases: ['golang'] },
  java:       { label: 'Java', svg: 'java.svg',       color: '#e76f00' },
  c:          { label: 'C', svg: 'c.svg',          color: '#a8b9cc', aliases: ['h'] },
  cpp:        { label: 'C++', svg: 'cplusplus.svg',  color: '#00599c', aliases: ['c++', 'cplusplus', 'cxx', 'cc', 'hpp'] },
  csharp:     { label: 'C#', svg: 'csharp.svg',     color: '#68217a', aliases: ['cs', 'c#'] },
  php:        { label: 'PHP', svg: 'php.svg',        color: '#777bb4' },
  swift:      { label: 'Swift', svg: 'swift.svg',      color: '#f05138' },
  kotlin:     { label: 'Kotlin', svg: 'kotlin.svg',     color: '#7f52ff', aliases: ['kt', 'kts'] },
  dart:       { label: 'Dart', svg: 'dart.svg',       color: '#0175c2' },
  scala:      { label: 'Scala', svg: 'scala.svg',      color: '#dc322f' },
  r:          { label: 'R', svg: 'r.svg',          color: '#276dc3' },
  elixir:     { label: 'Elixir', svg: 'elixir.svg',     color: '#9b6bb3', aliases: ['ex', 'exs'] },
  lua:        { label: 'Lua', svg: 'lua.svg',        color: '#7c83db', monochrome: true },
  html:       { label: 'HTML', svg: 'html5.svg',      color: '#e34f26', aliases: ['html5'] },
  css:        { label: 'CSS', svg: 'css3.svg',       color: '#1572b6', aliases: ['css3'] },
  sass:       { label: 'Sass', svg: 'sass.svg',       color: '#cc6699', aliases: ['scss'] },
  json:       { label: 'JSON', svg: 'json.svg',       color: '#a8b9cc', monochrome: true, aliases: ['jsonc', 'json5'] },
  yaml:       { label: 'YAML', svg: 'yaml.svg',       color: '#cb171e', aliases: ['yml'] },
  markdown:   { label: 'Markdown', svg: 'markdown.svg',   color: '#8bb9de', monochrome: true, aliases: ['md'] },
  powershell: { label: 'PowerShell', svg: 'powershell.svg', color: '#5391fe', aliases: ['ps1', 'pwsh'] }
};
const codeLanguageLookup = new Map();
Object.entries(codeLanguageIcons).forEach(([language, entry]) => {
  [language, ...(entry.aliases || [])].forEach(alias => codeLanguageLookup.set(alias, entry));
});
// Resolve from the script URL so versioned/localised pages and base paths work.
const codeLanguageIconBase = new URL('icons/devicon/', document.currentScript.src);

// Copy controls live outside <pre>, keeping labels out of copied code.
document.querySelectorAll('.content pre:not(.mermaid)').forEach(pre => {
  let figure = pre.parentElement;
  if (!figure.matches('figure.code-block')) {
    figure = document.createElement('figure');
    figure.className = 'code-block';
    pre.before(figure);
    figure.appendChild(pre);
  }
  let header = figure.querySelector('.code-block-header, figcaption');
  if (!header) {
    header = document.createElement('div');
    figure.prepend(header);
  }
  header.classList.add('code-block-header');
  const language = (pre.dataset.language ||
    (pre.querySelector('code')?.className.match(/(?:^|\s)language-([^\s]+)/) || [])[1] || '').toLowerCase();
  const entry = codeLanguageLookup.get(language);
  const icon = document.createElement('span');
  icon.className = 'code-language-icon';
  icon.setAttribute('aria-hidden', 'true');
  if (entry) {
    const url = new URL(entry.svg, codeLanguageIconBase).href;
    icon.style.setProperty('--code-language-color', entry.color);
    if (entry.monochrome) {
      icon.classList.add('code-language-monochrome');
      icon.style.setProperty('--code-language-mask', `url("${url}")`);
    } else {
      const image = document.createElement('img');
      image.src = url;
      image.alt = '';
      image.width = 16;
      image.height = 16;
      icon.appendChild(image);
    }
  } else {
    icon.innerHTML = '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m8 6-6 6 6 6m8-12 6 6-6 6m-3-15-2 18"/></svg>';
  }
  const title = header.querySelector('.code-block-title');
  if (title) {
    title.prepend(icon);
  } else {
    header.prepend(icon);
  }
  const controls = document.createElement('div');
  controls.className = 'code-block-controls';
  const languageName = document.createElement('span');
  languageName.className = 'code-block-language';
  languageName.textContent = entry?.label || language || 'Text';
  controls.appendChild(languageName);
  if (pre.classList.contains('diff-block')) {
    const badge = document.createElement('span');
    badge.className = 'code-block-diff-badge';
    badge.textContent = 'diff';
    controls.appendChild(badge);
  }
  header.appendChild(controls);

  if (pre.style.backgroundColor) {
    figure.classList.add('code-block-highlighted');
    figure.style.setProperty('--syntax-code-bg', getComputedStyle(pre).backgroundColor);
  }

  const copyIcon = '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>';
  const checkIcon = '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><polyline points="20 6 9 17 4 12"/></svg>';
  const btn = document.createElement('button');
  btn.type = 'button';
  btn.className = 'copy-btn';
  btn.setAttribute('aria-label', 'Copy code');
  btn.innerHTML = copyIcon + '<span aria-live="polite">Copy</span>';
  controls.appendChild(btn);
  let resetTimer;

  btn.addEventListener('click', async () => {
    const code = pre.querySelector('code');
    // Numbers and hidden-line labels use CSS pseudo-elements.
    const text = (code || pre).textContent;
    clearTimeout(resetTimer);
    try {
      await navigator.clipboard.writeText(text);
      btn.classList.add('copied');
      btn.innerHTML = checkIcon + '<span aria-live="polite">Copied!</span>';
    } catch (error) {
      btn.classList.remove('copied');
      btn.innerHTML = copyIcon + '<span aria-live="polite">Copy failed</span>';
    }
    resetTimer = setTimeout(() => {
      btn.classList.remove('copied');
      btn.innerHTML = copyIcon + '<span aria-live="polite">Copy</span>';
    }, 2000);
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

  // Long TOCs only show sub-headings for the section being read, so the
  // top-level outline stays visible without scrolling.
  const COLLAPSE_THRESHOLD = 12;
  const topTag = multiH1 ? 'H1' : 'H2';
  const collapsible = headings.length > COLLAPSE_THRESHOLD;

  const ul = document.createElement('ul');
  ul.className = 'toc-list' + (multiH1 ? ' toc-has-h1' : '') + (collapsible ? ' toc-collapsible' : '');
  let section = -1;
  headings.forEach(h => {
    const li = document.createElement('li');
    let cls = 'toc-item';
    if (h.tagName === 'H3') cls += ' toc-h3';
    else if (h.tagName === 'H2' && multiH1) cls += ' toc-h2';
    if (h.tagName === topTag) section++;
    else if (section >= 0) cls += ' toc-sub';
    li.className = cls;
    li.dataset.section = section;
    const a = document.createElement('a');
    a.href = '#' + h.id;
    a.textContent = h.textContent.replace(/\s*#$/, '');
    li.appendChild(a);
    ul.appendChild(li);
    if (cls.includes('toc-sub')) {
      ul.querySelector(`[data-section="${section}"]:not(.toc-sub)`).classList.add('toc-has-children');
    }
  });
  tocNav.appendChild(ul);

  const toc = tocNav.closest('.toc');
  function setActive(li) {
    tocNav.querySelectorAll('.toc-item').forEach(item => {
      item.classList.remove('active');
      item.classList.toggle('toc-open', item.dataset.section === li.dataset.section);
    });
    li.classList.add('active');
    // Keep the highlight in view when the TOC itself overflows.
    if (toc.scrollHeight > toc.clientHeight) {
      const top = li.offsetTop - toc.scrollTop;
      if (top < 0 || top + li.offsetHeight > toc.clientHeight) {
        toc.scrollTop = li.offsetTop - toc.clientHeight / 3;
      }
    }
  }

  const observer = new IntersectionObserver(entries => {
    entries.forEach(entry => {
      if (entry.isIntersecting) {
        const active = tocNav.querySelector(`a[href="#${entry.target.id}"]`);
        if (active) setActive(active.parentElement);
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

// Version switcher
(function() {
  var switcher = document.querySelector('.version-switcher');
  if (!switcher) return;

  var trigger = switcher.querySelector('.version-switcher-trigger');

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
  var searchIndexUrl = document.body.dataset.searchIndexUrl || (baseUrl + 'search-index.json');
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
      fetch(searchIndexUrl)
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

// API content types remain readable without JavaScript. Enhance each group
// independently with keyboard-accessible tabs; print CSS shows every panel.
document.querySelectorAll('[data-api-media]').forEach(group => {
  const controls = group.querySelector('.api-media-tabs');
  if (!controls) return;
  const tabs = Array.from(controls.querySelectorAll('[role="tab"]'));
  const panels = Array.from(group.querySelectorAll('[data-api-panel]'));
  if (!tabs.length || tabs.length !== panels.length) return;
  function select(index, focus) {
    tabs.forEach((tab, i) => {
      tab.setAttribute('aria-selected', String(i === index));
      tab.tabIndex = i === index ? 0 : -1;
      panels[i].hidden = i !== index;
      panels[i].setAttribute('role', 'tabpanel');
      panels[i].setAttribute('aria-labelledby', tab.id);
      panels[i].tabIndex = 0;
    });
    if (focus) tabs[index].focus();
  }
  tabs.forEach((tab, index) => {
    tab.addEventListener('click', () => select(index, false));
    tab.addEventListener('keydown', event => {
      let next;
      if (event.key === 'ArrowRight') next = (index + 1) % tabs.length;
      else if (event.key === 'ArrowLeft') next = (index - 1 + tabs.length) % tabs.length;
      else if (event.key === 'Home') next = 0;
      else if (event.key === 'End') next = tabs.length - 1;
      else return;
      event.preventDefault();
      select(next, true);
    });
  });
  function revealFragment() {
    let target;
    try { target = document.getElementById(decodeURIComponent(location.hash.slice(1))); } catch (_) { return; }
    const index = panels.findIndex(panel => panel === target || panel.contains(target));
    if (index >= 0) select(index, false);
  }
  select(0, false);
  controls.hidden = false;
  revealFragment();
  window.addEventListener('hashchange', revealFragment);
});

// Show "last updated" dates in the page's language
document.querySelectorAll('.last-updated time[datetime]').forEach(time => {
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(time.getAttribute('datetime'));
  if (!match) return;
  try {
    // Built from parts so it's a local date; new Date('YYYY-MM-DD') is UTC and can show the day before
    const date = new Date(Number(match[1]), Number(match[2]) - 1, Number(match[3]));
    const lang = (document.documentElement.lang || '').replace(/_/g, '-') || undefined;
    time.textContent = new Intl.DateTimeFormat(lang, { dateStyle: 'long' }).format(date);
  } catch (e) {
    // Keep the ISO date
  }
});
