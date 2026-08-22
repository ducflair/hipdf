/**
 * hipdf - Clean Vanilla Viewer for Showcase & Regression Reports
 */

if (window.pdfjsLib) {
  pdfjsLib.GlobalWorkerOptions.workerSrc = 'https://cdnjs.cloudflare.com/ajax/libs/pdf.js/3.11.174/pdf.worker.min.js';
}

class HipdfViewer {
  constructor() {
    this.data = null;
    this.selectedIndex = 0;
    this.currentPage = 1;
    this.zoom = 1.0;
    this.mode = 'side-by-side';
    this.filter = 'all';
    this.search = '';

    this.baseDoc = null;
    this.candDoc = null;
    this.prodDoc = null;
    this.isSyncing = false;

    this.init();
  }

  async init() {
    this.bindDom();
    this.bindEvents();

    try {
      const res = await fetch('results.json');
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      this.data = await res.json();
      this.setupHeader();
      this.renderFileList();

      let initial = 0;
      if (this.data.mode === 'comparison') {
        const firstChanged = this.data.files.findIndex(f => f.status === 'changed');
        if (firstChanged !== -1) initial = firstChanged;
      }
      this.selectFile(initial);
    } catch (e) {
      console.error('Failed to load results.json', e);
      this.currentFileName.textContent = 'Error loading results.json: ' + e.message;
    }
  }

  bindDom() {
    this.headerTag = document.getElementById('headerTag');
    this.headerSummary = document.getElementById('headerSummary');
    this.searchInput = document.getElementById('searchInput');
    this.filterTabs = document.getElementById('filterTabs');
    this.fileList = document.getElementById('fileList');

    this.countAll = document.getElementById('countAll');
    this.countChanged = document.getElementById('countChanged');
    this.countUnchanged = document.getElementById('countUnchanged');

    this.fileStatusBadge = document.getElementById('fileStatusBadge');
    this.currentFileName = document.getElementById('currentFileName');
    this.currentFileMeta = document.getElementById('currentFileMeta');
    this.modeTabs = document.getElementById('modeTabs');

    this.pageNavGroup = document.getElementById('pageNavGroup');
    this.prevPageBtn = document.getElementById('prevPageBtn');
    this.nextPageBtn = document.getElementById('nextPageBtn');
    this.pageCurrent = document.getElementById('pageCurrent');
    this.pageTotal = document.getElementById('pageTotal');

    this.zoomOutBtn = document.getElementById('zoomOutBtn');
    this.zoomInBtn = document.getElementById('zoomInBtn');
    this.zoomLabel = document.getElementById('zoomLabel');

    this.downloadBaseBtn = document.getElementById('downloadBaseBtn');
    this.downloadCandBtn = document.getElementById('downloadCandBtn');

    this.showcaseViewport = document.getElementById('showcaseViewport');
    this.showcasePages = document.getElementById('showcasePages');

    this.comparisonViewport = document.getElementById('comparisonViewport');
    this.diffViewport = document.getElementById('diffViewport');
    this.detailsViewport = document.getElementById('detailsViewport');

    this.baseScroll = document.getElementById('baseScroll');
    this.candScroll = document.getElementById('candScroll');
    this.baseCanvas = document.getElementById('baseCanvas');
    this.candCanvas = document.getElementById('candCanvas');
    this.baseEmpty = document.getElementById('baseEmpty');
    this.candEmpty = document.getElementById('candEmpty');
    this.baseLabel = document.getElementById('baseLabel');
    this.candLabel = document.getElementById('candLabel');

    this.diffImage = document.getElementById('diffImage');
    this.diffNoneNotice = document.getElementById('diffNoneNotice');

    this.detailsTable = document.getElementById('detailsTable');
    this.pagesTableBody = document.getElementById('pagesTableBody');
  }

  bindEvents() {
    this.searchInput.addEventListener('input', (e) => {
      this.search = e.target.value.toLowerCase().trim();
      this.renderFileList();
    });

    this.filterTabs.addEventListener('click', (e) => {
      const btn = e.target.closest('.filter-tab');
      if (!btn) return;
      this.filterTabs.querySelectorAll('.filter-tab').forEach(b => b.classList.remove('active'));
      btn.classList.add('active');
      this.filter = btn.dataset.filter;
      this.renderFileList();
    });

    this.modeTabs.addEventListener('click', (e) => {
      const tab = e.target.closest('.mode-tab');
      if (!tab) return;
      this.modeTabs.querySelectorAll('.mode-tab').forEach(t => t.classList.remove('active'));
      tab.classList.add('active');
      this.setMode(tab.dataset.mode);
    });

    this.prevPageBtn.addEventListener('click', () => this.setPage(this.currentPage - 1));
    this.nextPageBtn.addEventListener('click', () => this.setPage(this.currentPage + 1));

    this.zoomOutBtn.addEventListener('click', () => this.setZoom(this.zoom - 0.2));
    this.zoomInBtn.addEventListener('click', () => this.setZoom(this.zoom + 0.2));

    // Synchronize scroll in comparison mode
    this.baseScroll.addEventListener('scroll', () => {
      if (this.isSyncing) return;
      this.isSyncing = true;
      this.candScroll.scrollTop = this.baseScroll.scrollTop;
      this.candScroll.scrollLeft = this.baseScroll.scrollLeft;
      requestAnimationFrame(() => { this.isSyncing = false; });
    });

    this.candScroll.addEventListener('scroll', () => {
      if (this.isSyncing) return;
      this.isSyncing = true;
      this.baseScroll.scrollTop = this.candScroll.scrollTop;
      this.baseScroll.scrollLeft = this.candScroll.scrollLeft;
      requestAnimationFrame(() => { this.isSyncing = false; });
    });
  }

  setupHeader() {
    const s = this.data.summary;
    const isProduction = this.data.mode === 'production';
    const meta = this.data.meta || {};

    if (isProduction) {
      // Production Showcase Mode
      document.title = 'hipdf - Production Showcase';
      this.headerTag.style.display = 'none';
      this.headerSummary.innerHTML = `
        <span class="stat-item success">✓ ${s.total_files} Production PDFs (${s.total_pages} total pages)</span>
      `;
      // Hide comparison-only controls
      this.modeTabs.style.display = 'none';
      this.pageNavGroup.style.display = 'none';
      this.filterTabs.style.display = 'none';
      this.downloadBaseBtn.style.display = 'none';
      this.downloadCandBtn.textContent = 'Download PDF';

      // Show showcase viewport
      this.showcaseViewport.style.display = 'flex';
      this.comparisonViewport.style.display = 'none';
      this.diffViewport.style.display = 'none';
      this.detailsViewport.style.display = 'none';
    } else {
      // Regression Comparison Mode
      const prText = meta.pr_number ? `PR #${meta.pr_number}` : `${meta.base_ref || 'main'} → ${meta.head_ref || 'PR'}`;
      document.title = `hipdf Regression — ${prText}`;
      this.headerTag.textContent = prText;
      this.headerTag.style.display = 'inline-block';

      this.headerSummary.innerHTML = `
        <span class="stat-item success">✓ ${s.unchanged} unchanged</span>
        <span class="stat-item ${s.changed > 0 ? 'warning' : ''}">⚠ ${s.changed} changed</span>
        ${s.added ? `<span class="stat-item info">+ ${s.added} added</span>` : ''}
        ${s.removed ? `<span class="stat-item danger">- ${s.removed} removed</span>` : ''}
      `;

      this.baseLabel.textContent = meta.base_ref || 'main';
      this.candLabel.textContent = meta.head_ref || `PR #${meta.pr_number || ''}`;

      this.modeTabs.style.display = 'flex';
      this.pageNavGroup.style.display = 'flex';
      this.filterTabs.style.display = 'flex';

      this.showcaseViewport.style.display = 'none';
      this.comparisonViewport.style.display = 'flex';
    }

    this.countAll.textContent = s.total_files;
    this.countChanged.textContent = s.changed;
    this.countUnchanged.textContent = s.unchanged;
  }

  renderFileList() {
    const isProduction = this.data.mode === 'production';
    const files = this.data.files
      .map((f, i) => ({ file: f, idx: i }))
      .filter(({ file }) => {
        if (!isProduction) {
          if (this.filter === 'changed' && file.status !== 'changed') return false;
          if (this.filter === 'unchanged' && file.status !== 'unchanged') return false;
        }
        if (this.search && !file.filename.toLowerCase().includes(this.search)) return false;
        return true;
      });

    this.fileList.innerHTML = '';
    files.forEach(({ file, idx }) => {
      const li = document.createElement('li');
      li.className = `file-item ${idx === this.selectedIndex ? 'active' : ''}`;
      
      const parts = file.filename.split('/');
      const name = parts.pop();
      const dir = parts.length ? parts.join('/') + '/' : '';
      const pages = file.candidate_pages || file.baseline_pages || file.pages || 1;

      if (isProduction) {
        li.innerHTML = `
          <div class="file-info">
            <div class="file-name-text">${name}</div>
            ${dir ? `<div class="file-sub-text">${dir}</div>` : ''}
          </div>
          <span class="file-badge">${pages}p</span>
        `;
      } else {
        const icon = file.status === 'unchanged' ? '✓' : (file.status === 'changed' ? '⚠' : (file.status === 'added' ? '+' : '−'));
        li.innerHTML = `
          <span class="file-icon ${file.status}">${icon}</span>
          <div class="file-info">
            <div class="file-name-text">${name}</div>
            ${dir ? `<div class="file-sub-text">${dir}</div>` : ''}
          </div>
          ${file.pages_with_diff > 0 ? `<span class="file-badge diff">${file.pages_with_diff} diff</span>` : ''}
          <span class="file-badge">${pages}p</span>
        `;
      }

      li.addEventListener('click', () => this.selectFile(idx));
      this.fileList.appendChild(li);
    });
  }

  async selectFile(idx) {
    if (!this.data.files[idx]) return;
    this.selectedIndex = idx;
    this.currentPage = 1;

    // Highlight active in list
    this.fileList.querySelectorAll('.file-item').forEach(el => el.classList.remove('active'));
    const items = this.fileList.children;
    for (let i = 0; i < items.length; i++) {
      if (items[i].textContent.includes(this.data.files[idx].filename.split('/').pop())) {
        items[i].classList.add('active');
        break;
      }
    }

    const file = this.data.files[idx];
    const isProduction = this.data.mode === 'production';

    this.currentFileName.textContent = file.filename;

    if (isProduction) {
      this.fileStatusBadge.style.display = 'none';
      const sizeStr = file.size ? `${(file.size / 1024).toFixed(1)} KB` : '';
      const pagesCount = file.pages || (file.pages_detail ? file.pages_detail.length : 1);
      this.currentFileMeta.textContent = `• ${sizeStr} • ${pagesCount} page${pagesCount > 1 ? 's' : ''}`;
      
      const pdfPath = file.production_path || file.candidate_path;
      this.downloadCandBtn.href = pdfPath;
      this.downloadCandBtn.style.display = 'inline-flex';
      this.downloadBaseBtn.style.display = 'none';

      await this.loadShowcasePdf(pdfPath);
    } else {
      this.fileStatusBadge.style.display = 'inline-block';
      this.fileStatusBadge.textContent = file.status === 'unchanged' ? '✓' : (file.status === 'changed' ? '⚠' : '+');
      this.fileStatusBadge.className = `status-indicator ${file.status}`;

      const bSize = file.baseline_size ? `${(file.baseline_size / 1024).toFixed(1)} KB` : 'N/A';
      const cSize = file.candidate_size ? `${(file.candidate_size / 1024).toFixed(1)} KB` : 'N/A';
      this.currentFileMeta.textContent = `• Base: ${bSize} → PR: ${cSize}`;

      if (file.baseline_path) {
        this.downloadBaseBtn.href = file.baseline_path;
        this.downloadBaseBtn.style.display = 'inline-flex';
      } else {
        this.downloadBaseBtn.style.display = 'none';
      }

      if (file.candidate_path) {
        this.downloadCandBtn.href = file.candidate_path;
        this.downloadCandBtn.textContent = 'Download PR';
        this.downloadCandBtn.style.display = 'inline-flex';
      } else {
        this.downloadCandBtn.style.display = 'none';
      }

      await this.loadComparisonPdfs(file);
      this.updateDetailsTable(file);
      this.renderComparisonPage();
    }
  }

  // Load and render all pages continuously for Showcase mode
  async loadShowcasePdf(pdfPath) {
    this.showcasePages.innerHTML = '<div class="empty-notice">Loading PDF...</div>';
    try {
      this.prodDoc = await pdfjsLib.getDocument(pdfPath).promise;
      this.showcasePages.innerHTML = '';

      for (let pageNum = 1; pageNum <= this.prodDoc.numPages; pageNum++) {
        const pageCard = document.createElement('div');
        pageCard.className = 'showcase-page-card';
        const canvas = document.createElement('canvas');
        pageCard.appendChild(canvas);

        if (this.prodDoc.numPages > 1) {
          const label = document.createElement('span');
          label.className = 'showcase-page-label';
          label.textContent = `Page ${pageNum} / ${this.prodDoc.numPages}`;
          pageCard.appendChild(label);
        }

        this.showcasePages.appendChild(pageCard);
        await this.renderSingleCanvas(this.prodDoc, pageNum, canvas);
      }
    } catch (e) {
      console.error('Failed to load showcase PDF:', e);
      this.showcasePages.innerHTML = `<div class="empty-notice">Error loading PDF: ${e.message}</div>`;
    }
  }

  async loadComparisonPdfs(file) {
    this.baseDoc = null;
    this.candDoc = null;
    const loads = [];

    if (file.baseline_path) {
      loads.push(pdfjsLib.getDocument(file.baseline_path).promise.then(d => { this.baseDoc = d; }).catch(() => {}));
    }
    if (file.candidate_path) {
      loads.push(pdfjsLib.getDocument(file.candidate_path).promise.then(d => { this.candDoc = d; }).catch(() => {}));
    }
    await Promise.all(loads);

    const total = Math.max(
      this.baseDoc ? this.baseDoc.numPages : 0,
      this.candDoc ? this.candDoc.numPages : 0,
      1
    );
    this.pageTotal.textContent = total;
    this.pageCurrent.textContent = this.currentPage;
  }

  async renderComparisonPage() {
    this.pageCurrent.textContent = this.currentPage;
    const total = parseInt(this.pageTotal.textContent, 10);
    this.prevPageBtn.disabled = this.currentPage <= 1;
    this.nextPageBtn.disabled = this.currentPage >= total;

    const file = this.data.files[this.selectedIndex];
    const pageMeta = file.pages && file.pages[this.currentPage - 1];

    if (this.mode === 'side-by-side') {
      await Promise.all([
        this.renderSingleCanvas(this.baseDoc, this.currentPage, this.baseCanvas, this.baseEmpty),
        this.renderSingleCanvas(this.candDoc, this.currentPage, this.candCanvas, this.candEmpty)
      ]);
    } else if (this.mode === 'diff') {
      if (pageMeta && pageMeta.diff_image) {
        this.diffImage.src = pageMeta.diff_image;
        this.diffImage.style.display = 'block';
        this.diffNoneNotice.style.display = 'none';
      } else {
        this.diffImage.style.display = 'none';
        this.diffNoneNotice.style.display = 'block';
      }
    }
  }

  async renderSingleCanvas(doc, pageNum, canvas, emptyEl = null) {
    if (!doc || pageNum > doc.numPages) {
      if (canvas) canvas.style.display = 'none';
      if (emptyEl) emptyEl.style.display = 'block';
      return;
    }

    if (emptyEl) emptyEl.style.display = 'none';
    if (canvas) canvas.style.display = 'block';

    const page = await doc.getPage(pageNum);
    const viewport = page.getViewport({ scale: this.zoom * 1.3 });
    const ratio = window.devicePixelRatio || 1;

    canvas.width = Math.floor(viewport.width * ratio);
    canvas.height = Math.floor(viewport.height * ratio);
    canvas.style.width = Math.floor(viewport.width) + 'px';
    canvas.style.height = Math.floor(viewport.height) + 'px';

    const ctx = canvas.getContext('2d');
    const transform = ratio !== 1 ? [ratio, 0, 0, ratio, 0, 0] : null;
    await page.render({ canvasContext: ctx, transform, viewport }).promise;
  }

  setMode(mode) {
    this.mode = mode;
    this.comparisonViewport.style.display = mode === 'side-by-side' ? 'flex' : 'none';
    this.diffViewport.style.display = mode === 'diff' ? 'flex' : 'none';
    this.detailsViewport.style.display = mode === 'details' ? 'block' : 'none';
    this.renderComparisonPage();
  }

  setPage(p) {
    const total = parseInt(this.pageTotal.textContent, 10);
    this.currentPage = Math.max(1, Math.min(total, p));
    this.renderComparisonPage();
  }

  async setZoom(z) {
    this.zoom = Math.max(0.4, Math.min(2.5, Math.round(z * 10) / 10));
    this.zoomLabel.textContent = `${Math.round(this.zoom * 100)}%`;

    if (this.data.mode === 'production') {
      if (this.prodDoc) {
        const cards = this.showcasePages.querySelectorAll('.showcase-page-card canvas');
        for (let i = 0; i < cards.length; i++) {
          await this.renderSingleCanvas(this.prodDoc, i + 1, cards[i]);
        }
      }
    } else {
      this.renderComparisonPage();
    }
  }

  updateDetailsTable(file) {
    this.detailsTable.innerHTML = `
      <tr><th style="width:160px">File</th><td><code>${file.filename}</code></td></tr>
      <tr><th>Status</th><td>${file.status}</td></tr>
      <tr><th>Baseline Size</th><td>${file.baseline_size ? file.baseline_size + ' bytes' : 'N/A'}</td></tr>
      <tr><th>Candidate Size</th><td>${file.candidate_size ? file.candidate_size + ' bytes' : 'N/A'}</td></tr>
      <tr><th>Baseline SHA-256</th><td><code>${file.baseline_sha256 || 'N/A'}</code></td></tr>
      <tr><th>Candidate SHA-256</th><td><code>${file.candidate_sha256 || 'N/A'}</code></td></tr>
    `;

    const pages = file.pages || file.pages_detail || [];
    this.pagesTableBody.innerHTML = '';
    pages.forEach(p => {
      const tr = document.createElement('tr');
      tr.innerHTML = `
        <td>Page ${p.page_number}</td>
        <td>${p.status}</td>
        <td>${p.width} × ${p.height} pt</td>
        <td>${p.diff_pixels.toLocaleString()}</td>
        <td>${p.diff_percent}%</td>
      `;
      this.pagesTableBody.appendChild(tr);
    });
  }
}

document.addEventListener('DOMContentLoaded', () => {
  new HipdfViewer();
});
