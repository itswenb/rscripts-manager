if(!window._monacoLoaded){
window._monacoLoaded=true;
(function(){
    var s1=document.createElement('script');s1.src='/static/vendor/monaco/min/vs/loader.js';
    s1.onload=function(){var s2=document.createElement('script');s2.src='/static/vendor/monaco/monaco.js';document.head.appendChild(s2);};
    document.head.appendChild(s1);
})();
}

var currentFilePath = '';
var extLangMap = {r:'r',rmd:'markdown',json:'json',yaml:'yaml',yml:'yaml',toml:'toml',py:'python',js:'javascript',ts:'typescript',html:'html',css:'css',sql:'sql',sh:'shell',md:'markdown',xml:'xml',csv:'plaintext',tsv:'plaintext',txt:'plaintext',log:'plaintext',ini:'ini',cfg:'ini',conf:'ini',env:'plaintext',gitignore:'plaintext',bash:'shell'};

async function openFile(path) {
    const res = await fetch('/files/read?path=' + encodeURIComponent(path));
    if (!res.ok) return;
    const data = await res.json();
    const modal = document.getElementById('file-modal');
    const img = document.getElementById('modal-image');
    const pdf = document.getElementById('modal-pdf');
    const editorWrap = document.getElementById('editor-wrap');
    const unsupported = document.getElementById('modal-unsupported');
    const saveBtn = document.getElementById('modal-save');

    modal.label = data.filename;
    img.classList.add('hidden');
    pdf.classList.add('hidden');
    pdf.removeAttribute('src');
    editorWrap.innerHTML = '';
    unsupported.classList.add('hidden');
    saveBtn.classList.add('hidden');
    currentFilePath = path;

    if (data.image) {
        img.src = '/files/download?path=' + encodeURIComponent(path);
        img.classList.remove('hidden');
    } else if (data.pdf) {
        pdf.src = '/files/download?path=' + encodeURIComponent(path);
        pdf.classList.remove('hidden');
    } else if (data.editable) {
        const ext = (data.filename.split('.').pop() || '').toLowerCase();
        const lang = extLangMap[ext] || 'plaintext';
        const el = document.createElement('monaco-editor');
        el.setAttribute('language', lang);
        el.setAttribute('defaultvalue', data.content);
        el.style.width = '100%';
        el.style.height = '100%';
        editorWrap.appendChild(el);
        saveBtn.classList.remove('hidden');
    } else {
        unsupported.classList.remove('hidden');
    }
    window.showShoelaceDialog(modal);
}

if (!document.getElementById('modal-save')._bound) {
document.getElementById('modal-save')._bound = true;
document.getElementById('modal-save').addEventListener('click', async function() {
    const el = document.querySelector('#editor-wrap monaco-editor');
    const content = el?.editor ? el.editor.getValue() : '';
    await fetch('/files/save', {
        method: 'POST',
        headers: {'Content-Type': 'application/json'},
        body: JSON.stringify({path: currentFilePath, content})
    });
    this.textContent = '已保存';
    setTimeout(() => { this.textContent = '保存'; }, 1500);
});

document.getElementById('file-modal').addEventListener('sl-after-hide', function() {
    const el = document.querySelector('#editor-wrap monaco-editor');
    if (el?.editor) { el.editor.dispose(); }
    document.getElementById('editor-wrap').innerHTML = '';
    document.getElementById('modal-pdf').removeAttribute('src');
});
}

function getCurrentPath() {
    return new URLSearchParams(window.location.search).get('path') || '';
}

function promptNewFolder() {
    const name = prompt('文件夹名称：');
    if (!name) return;
    fetch('/files/mkdir', {
        method: 'POST',
        headers: {'Content-Type': 'application/x-www-form-urlencoded'},
        body: new URLSearchParams({path: getCurrentPath(), name})
    }).then(() => {
        htmx.ajax('GET', '/files?path=' + encodeURIComponent(getCurrentPath()), {target: '#file-content', swap: 'innerHTML'});
    });
}

function promptNewFile() {
    const name = prompt('文件名称：');
    if (!name) return;
    const path = getCurrentPath() ? getCurrentPath() + '/' + name : name;
    fetch('/files/save', {
        method: 'POST',
        headers: {'Content-Type': 'application/json'},
        body: JSON.stringify({path, content: ''})
    }).then(() => {
        htmx.ajax('GET', '/files?path=' + encodeURIComponent(getCurrentPath()), {target: '#file-content', swap: 'innerHTML'});
    });
}

function promptRename(oldPath, oldName) {
    const newName = prompt('重命名为：', oldName);
    if (!newName || newName === oldName) return;
    fetch('/files/rename', {
        method: 'POST',
        headers: {'Content-Type': 'application/x-www-form-urlencoded'},
        body: new URLSearchParams({path: oldPath, new_name: newName})
    }).then(() => {
        htmx.ajax('GET', '/files?path=' + encodeURIComponent(getCurrentPath()), {target: '#file-content', swap: 'innerHTML'});
    });
}

function confirmDelete(path) {
    if (!confirm('确定删除？')) return;
    fetch('/files/delete', {
        method: 'POST',
        headers: {'Content-Type': 'application/x-www-form-urlencoded'},
        body: new URLSearchParams({path})
    }).then(() => {
        htmx.ajax('GET', '/files?path=' + encodeURIComponent(getCurrentPath()), {target: '#file-content', swap: 'innerHTML'});
    });
}

if (!window._fileDragInit) {
window._fileDragInit = true;
(function() {
    const CHUNK_SIZE = 64 * 1024 * 1024;
    const page = document.getElementById('file-page');
    const overlay = document.getElementById('drop-overlay');
    const progress = document.getElementById('upload-progress');
    const bar = document.getElementById('upload-bar');
    const pct = document.getElementById('upload-pct');
    const fname = document.getElementById('upload-filename');

    async function uploadAndRefresh(files) {
        for (const file of files) {
            await uploadFile(file);
        }
        htmx.ajax('GET', '/files?path=' + encodeURIComponent(getCurrentPath()), {target: '#file-content', swap: 'innerHTML'});
    }

    window.handleFileInput = function(input) {
        if (input.files.length) {
            uploadAndRefresh(input.files);
            input.value = '';
        }
    };
    let dragCount = 0;

    function getCurrentPath() {
        const params = new URLSearchParams(window.location.search);
        return params.get('path') || '';
    }

    page.addEventListener('dragenter', function(e) {
        e.preventDefault();
        dragCount++;
        overlay.classList.remove('hidden');
    });
    page.addEventListener('dragleave', function(e) {
        e.preventDefault();
        dragCount--;
        if (dragCount <= 0) { overlay.classList.add('hidden'); dragCount = 0; }
    });
    page.addEventListener('dragover', function(e) { e.preventDefault(); });
    page.addEventListener('drop', async function(e) {
        e.preventDefault();
        dragCount = 0;
        overlay.classList.add('hidden');
        uploadAndRefresh(e.dataTransfer.files);
    });

    async function uploadFile(file) {
        fname.textContent = file.name;
        progress.classList.remove('hidden');
        bar.style.width = '0%';
        pct.textContent = '0%';

        if (file.size <= CHUNK_SIZE) {
            const fd = new FormData();
            fd.append('path', getCurrentPath());
            fd.append('file', file);
            await fetch('/files/upload', { method: 'POST', body: fd });
            bar.style.width = '100%';
            pct.textContent = '100%';
        } else {
            const total = Math.ceil(file.size / CHUNK_SIZE);
            for (let i = 0; i < total; i++) {
                const start = i * CHUNK_SIZE;
                const chunk = file.slice(start, start + CHUNK_SIZE);
                const fd = new FormData();
                fd.append('path', getCurrentPath());
                fd.append('filename', file.name);
                fd.append('offset', start.toString());
                fd.append('chunk', chunk);
                await fetch('/files/upload-chunk', { method: 'POST', body: fd });
                const p = Math.round(((i + 1) / total) * 100);
                bar.style.width = p + '%';
                pct.textContent = p + '%';
            }
        }
        setTimeout(() => { progress.classList.add('hidden'); }, 1500);
    }
})();
}
