/* ST IDE: edición local, coloreado, sugerencias y control de ejecución. */
'use strict';
/** Busca un elemento por su identificador compartido con index.html. */
const $ = id => document.getElementById(id);
const editor = $('editor');
let folded = new FoldDocument();
let inputAnchor = Infinity;
/** Guardar y ejecutar siempre consultan el documento íntegro, no la proyección. */
const fullText = () => folded.text;
// El vocabulario llega del lexer Rust; no hay una segunda lista de textos ST.
let keywords = [], snippets = {}, keywordInfo = new Map(), declarationWords = [];
let languageVersion = '', languageError = '';
let standardModules = {};
// Python conserva su vocabulario independiente; ST sigue leyendo KEYWORDS de Rust.
let pythonWords = [], pythonTimer;
const pythonSnippets = {
  def: 'def nombre(argumento):\n    pass',
  class: 'class Nombre:\n    def __init__(self):\n        pass',
  if: 'if condicion:\n    pass',
  for: 'for elemento in elementos:\n    pass',
  while: 'while condicion:\n    pass',
  try: 'try:\n    pass\nexcept Exception as error:\n    pass',
  with: 'with open("archivo.txt") as archivo:\n    pass',
  match: 'match valor:\n    case patron:\n        pass'
};

/** Carga nombres del Python del servidor, sin importar paquetes del proyecto. */
async function loadPython() {
  try {
    const data = await api('python-language');
    pythonWords = [...new Set([...data.keywords, ...data.builtins])];
  } catch (error) { message('Vocabulario Python: ' + error.message); }
}

/** Agrupa pulsaciones y aplica tokens solo si el archivo y su contenido siguen vigentes. */
function schedulePythonHighlight(text) {
  clearTimeout(pythonTimer);
  const path = current;
  pythonTimer = setTimeout(async () => {
    try {
      const data = await api('python-highlight', {text});
      if (current !== path || editor.value !== text) return;
      const categories = new Set(['keyword', 'string', 'comment', 'number', 'boolean', 'type', 'function', 'variable']);
      $('highlight').innerHTML = data.pieces.map(piece => {
        const value = escapeHtml(piece.text);
        return categories.has(piece.category) ? `<span class="${piece.category}">${value}</span>` : value;
      }).join('') + '\n';
      syncScroll();
    } catch (error) {
      // El texto plano permanece visible si falla la conexión; la edición continúa.
      if (current === path && editor.value === text) $('status').textContent = 'Colores Python sin conexión';
    }
  }, 180);
}


/** Genera plantillas por significado usando las grafías actuales del diccionario. */
function buildSnippets(entries) {
  const spellings = new Map();
  entries.forEach(item => { if (!spellings.has(item.kind)) spellings.set(item.kind, item.word); });
  const result = {};
  for (const item of entries) {
    const w = kind => kind === item.kind ? item.word : spellings.get(kind);
    const rules = {
      Funcion: [['Entero','Retornar'], () => `${w('Funcion')} nombre(a: ${w('Entero')}) -> ${w('Entero')} {\n    ${w('Retornar')} a;\n}`],
      Si: [['Verdadero'], () => `${w('Si')} ${w('Verdadero')} {\n    \n}`],
      Mientras: [['Verdadero','Romper'], () => `${w('Mientras')} ${w('Verdadero')} {\n    ${w('Romper')};\n}`],
      Para: [['En','Rango','Mostrar'], () => `${w('Para')} i ${w('En')} ${w('Rango')}(10) {\n    ${w('Mostrar')}(i);\n}`],
      Mostrar: [[], () => `${item.word}();`],
      Variable: [['Entero'], () => `${item.word} nombre: ${w('Entero')} = 0;`],
      Coincidir: [['Caso','Sino'], () => `${item.word} valor {\n    ${w('Caso')} 1 {\n        \n    }\n    ${w('Sino')} {\n        \n    }\n}`],
      Intentar: [['Capturar','Finalmente'], () => `${item.word} {\n    \n} ${w('Capturar')} error {\n    \n} ${w('Finalmente')} {\n    \n}`],
      Clase: [['Variable','Cadena'], () => `${item.word} Persona {\n    ${w('Variable')} nombre: ${w('Cadena')};\n}`],
      Estructura: [['Variable','Entero'], () => `${item.word} Punto {\n    ${w('Variable')} x: ${w('Entero')};\n    ${w('Variable')} z: ${w('Entero')};\n}`],
      Implementar: [['Funcion'], () => `${item.word} Persona {\n    ${w('Funcion')} saludar() {\n        \n    }\n}`],
      Desde: [['Importar'], () => `${item.word} \"modulo.st\" ${w('Importar')} nombre;`],
      Importar: [[], () => `${item.word} "modulo.st";`],
      Entrada: [[], () => `${item.word}("Ingresa tu nombre: ")`],
      Abrir: [[], () => `${item.word}(\"archivo.txt\")`],
      Escribir: [[], () => `${item.word}(\"archivo.txt\", texto);`],
      Buscar: [[], () => `${item.word}(texto, \"patrón\")`],
      Reemplazar: [[], () => `${item.word}(texto, \"patrón\", \"nuevo\")`],
      Interpolar: [[], () => `${item.word}(\"Hola {}\", nombre)`],
      Agregar: [[], () => `${item.word}(valores, 0);`],
      Longitud: [[], () => `${item.word}(valores)`]
    };
    if (item.kind === 'Contextual' && item.word === 'usar') {
      result[item.word] = 'usar recurso = recursos.buffer(1024) {\n    \n}';
    }
    const rule = rules[item.kind];
    if (rule && rule[0].every(k => spellings.has(k))) result[item.word] = rule[1]();
  }
  return result;
}

/** Detecta cambios en KEYWORDS sin recargar la página ni descartar la edición. */
async function refreshLanguage() {
  try {
    const data = await api('language');
    if (data.version !== languageVersion) {
      if (languageVersion) message('Vocabulario actualizado. Recompila PyForge para usar los cambios al ejecutar.');
      languageVersion = data.version;
      keywordInfo = new Map(data.keywords.map(item => [item.word, item]));
      keywords = [...data.keywords.map(item => item.word), ...(data.builtins || [])];
      standardModules = data.modules || {};
      declarationWords = data.keywords.filter(item => ['Funcion','Variable'].includes(item.kind)).map(item => item.word);
      snippets = {...buildSnippets(data.keywords), tipo: "tipo(valor)", entero: "entero(valor)", flotante: "flotante(valor)", booleano: "booleano(valor)", mixto: "mixto"};
      painted = null; hideCompletion(); render();
    }
    languageError = '';
  } catch (error) {
    if (languageError !== error.message) message('Vocabulario: ' + error.message);
    languageError = error.message;
  }
  setTimeout(refreshLanguage, 1500);
}
// Caché de pintado y estado de edición; original permite detectar cambios pendientes.
let painted = null, paintedPath = null;
let inputReady = false, sendingInput = false;
let readOnly = false;
let current = '', original = '', running = false, options = [], choice = 0, timer, saving = false;
/** Compara el contenido actual con la última versión leída o guardada. */
const dirty = () => current && fullText() !== original;

/** Coincide con la validación del servidor; ambos formatos usan pyforgec. */
const canRun = () => current.endsWith('.st') || current.endsWith('.py');

/** Envía solicitudes autenticadas al servidor local y presenta errores claros. */
async function api(route, body = {}) {
  const response = await fetch('/api/' + route, {method:'POST', headers:{'Content-Type':'application/json','X-ST-Token':document.querySelector('meta[name="st-token"]').content}, body:JSON.stringify(body)});
  const data = await response.json();
  if (!response.ok) throw new Error(data.error || 'No se pudo completar la operación.');
  return data;
}
/** Muestra un aviso temporal y sustituye el temporizador del aviso anterior. */
function message(text) {
  $('toast').textContent = text; $('toast').hidden = false;
  clearTimeout(timer); timer = setTimeout(() => $('toast').hidden = true, 6500);
}
/** Escapa código del usuario antes de insertarlo en la capa HTML de colores. */
const escapeHtml = text => text.replace(/[&<>"']/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));

/** Un único recorrido evita colorear palabras dentro de cadenas y comentarios. */
function colorize(text) {
  if (current.endsWith('.py')) { schedulePythonHighlight(text); return escapeHtml(text); }
  const format = current.split('.').pop().toLowerCase();
  if (['html', 'css', 'js'].includes(format)) return WebLanguages.highlight(text, format);
  if (['json', 'toml', 'db'].includes(format)) return FileFormats.highlight(text, format === 'db' ? 'sql' : format);
  if (!current.endsWith('.st')) return escapeHtml(text);
  const regex = /\/\*[\s\S]*?(?:\*\/|$)|"""[\s\S]*?(?:"""|$)|\/\/[^\n]*|#[^\n]*|"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'|\b\d+(?:\.\d+)?(?:[eE][+-]?\d+)?\b|\b[A-Za-z_][A-Za-z_0-9]*\b/g;
  let result = '', last = 0;
  for (const match of text.matchAll(regex)) {
    result += escapeHtml(text.slice(last, match.index));
    const word = match[0]; let cls = '';
    if (word.startsWith('//') || word.startsWith('#') || word.startsWith('/*') || word.startsWith('\"\"\"')) cls = 'comment';
    else if (/^["']/.test(word)) cls = 'string';
    else if (/^\d/.test(word)) cls = 'number';
    else if (keywordInfo.has(word)) cls = keywordInfo.get(word).category;
    else if (/^\s*\(/.test(text.slice(match.index + word.length))) cls = 'function';
    else cls = 'variable';
    result += cls ? `<span class="${cls}">${escapeHtml(word)}</span>` : escapeHtml(word);
    last = match.index + word.length;
  }
  return result + escapeHtml(text.slice(last));
}
/** Desplaza colores y números de línea junto al textarea, sin mover el cursor. */
function syncScroll() {
  $('highlight').scrollTop = editor.scrollTop;
  $('highlight').scrollLeft = editor.scrollLeft;
  $('lines').scrollTop = editor.scrollTop;
  positionCompletion();
}
/** Repinta solo si cambia el texto o la ruta; actualiza botones y posición siempre. */
function render() {
  if (painted !== editor.value || paintedPath !== current) {
    $('highlight').innerHTML = colorize(editor.value) + '\n';
    renderFoldLines();
    painted = editor.value; paintedPath = current;
  }
  $('dirty').textContent = dirty() ? '●' : '';
  $('status').textContent = current ? (readOnly ? 'SQLite · solo lectura' : dirty() ? 'Cambios sin guardar' : 'Guardado') : 'Sin archivo abierto';
  $('save').disabled = !current || saving || readOnly;
  $('check').disabled = !canRun() || running || saving || readOnly;
  $('search-open').disabled = !current;
  $('delete').disabled = !current || saving || running || readOnly;
  $('run').disabled = !canRun() || running || saving;
  $('stop').disabled = !running;
  $('terminal-line').disabled = !inputReady || sendingInput;
  $('terminal-send').disabled = !inputReady || sendingInput;
  $('terminal-eof').disabled = !inputReady || sendingInput;
  $('terminal-line').placeholder = inputReady ? 'Escribe una respuesta y pulsa Enter…' : 'Ejecuta un programa para responder aquí…';
  const before = fullText().slice(0, folded.offset(editor.selectionStart)).split('\n');
  $('unfold').disabled = !folded.folds.length;
  $('position').textContent = `Lín ${before.length}, Col ${before.at(-1).length + 1}`;
  syncScroll();
}

/** Explorador agrupado por carpetas; nombres de archivo se insertan como texto. */
async function refresh() {
  const data = await api('tree');
  $('lock-project').hidden = !data.protected;
  $('project').textContent = '▾ ' + data.name;
  $('tree').replaceChildren();
  const folders = new Map([['', $('tree')]]);
  for (const path of data.files) {
    const parts = path.split('/'); let parent = '';
    for (const folder of parts.slice(0,-1)) {
      const key = parent ? parent + '/' + folder : folder;
      if (!folders.has(key)) {
        const details = document.createElement('details'); details.open = true;
        const summary = document.createElement('summary'); summary.textContent = folder;
        details.append(summary); folders.get(parent).append(details); folders.set(key, details);
      }
      parent = key;
    }
    const button = document.createElement('button');
    if (path.endsWith('.st') || path.endsWith('.py')) { const icon = document.createElement('img'); icon.src=path.endsWith('.py')?'/python.svg':'/st.svg'; icon.alt=path.endsWith('.py')?'Python':'ST'; button.append(icon); }
    else { const icon = document.createElement('span'); icon.textContent='◇'; button.append(icon); }
    button.append(document.createTextNode(parts.at(-1))); button.title = path;
    button.classList.toggle('active', path === current);
    button.onclick = () => openFile(path).catch(e => message(e.message));
    folders.get(parent).append(button);
  }
  return data.initial_file;
}
let opening = 0;
/** Abre un archivo con protección de cambios y descarta respuestas de aperturas antiguas. */
async function openFile(path) {
  if (dirty() && !confirm('Hay cambios sin guardar. ¿Descartarlos y abrir otro archivo?')) return;
  const request = ++opening;
  const data = await api('read', {path});
  if (request !== opening) return;
  readOnly = Boolean(data.readonly); editor.readOnly = readOnly;
  current = path; original = data.text; folded = new FoldDocument(data.text, path.split('.').pop()); editor.value = data.text; editor.disabled = false;
  editor.scrollTop = editor.scrollLeft = 0;
  $('welcome').hidden = true; $('filename').textContent = path.split('/').at(-1);
  $('breadcrumb').textContent = path; $('language').textContent = path.endsWith('.st') ? 'ST' : path.endsWith('.py') ? 'Python' : ({html:'HTML', css:'CSS', js:'JavaScript', json:'JSON', toml:'TOML', db:'SQLite'}[path.split('.').pop().toLowerCase()] || 'Texto');
  hideCompletion(); render(); editor.focus(); await refresh();
}
/** Guarda con la versión original para detectar conflictos; restaura el estado aunque falle. */
async function save() {
  if (!current || saving || readOnly) return false;
  const path = current, text = fullText(), baseline = original;
  saving = true; render();
  try {
    await api('save', {path, text, original:baseline});
    if (current === path) original = text;
    return true;
  } finally { saving = false; render(); }
}

/** Comprueba sintaxis y tipos sin crear ni ejecutar el programa. */
async function checkCurrent() {
  if (running || !canRun() || readOnly) return;
  const path = current;
  if (!await save()) return;
  $('execution').textContent = 'COMPROBANDO';
  const result = await api('check', {path});
  cleared = '';
  lastOutput = result.output;
  $('output').textContent = result.output;
  $('output').scrollTop = $('output').scrollHeight;
  $('execution').textContent = result.ok ? 'SIN ERRORES' : 'CON ERRORES';
  message(result.ok ? 'Comprobación terminada sin errores.' : 'Revisa el diagnóstico de la salida.');
}

/** Abre la búsqueda sobre el documento completo; los bloques se despliegan para
 * que una coincidencia nunca permanezca invisible detrás de un pliegue.
 */
function openSearch(withReplacement = false) {
  if (!current) return;
  const position = folded.offset(editor.selectionStart);
  folded.folds = [];
  showFolded(position);
  $('search-panel').hidden = false;
  $('replace-text').hidden = !withReplacement;
  $('replace-one').hidden = !withReplacement;
  $('replace-all').hidden = !withReplacement;
  $('search-text').focus();
  $('search-text').select();
}

/** Selecciona la siguiente coincidencia, volviendo al inicio/final al alcanzarlo. */
function findText(direction = 1) {
  const query = $('search-text').value;
  const start = direction < 0 ? editor.selectionStart : editor.selectionEnd;
  const match = EditorSearch.find(fullText(), query, start, direction, $('search-case').checked);
  $('search-status').textContent = match ? `${match.start + 1} · encontrado` : 'Sin coincidencias';
  if (match) {
    editor.focus();
    editor.setSelectionRange(match.start, match.end);
    const line = fullText().slice(0, match.start).split('\n').length;
    editor.scrollTop = Math.max(0, (line - 3) * 24);
    render();
  }
  return match;
}

/** Sustituye la selección actual si coincide; en caso contrario busca primero. */
function replaceText() {
  const query = $('search-text').value;
  const selected = fullText().slice(editor.selectionStart, editor.selectionEnd);
  const equal = $('search-case').checked ? selected === query : selected.toLocaleLowerCase('es') === query.toLocaleLowerCase('es');
  if (!query) return;
  if (!equal) { findText(1); return; }
  insert($('replace-text').value, editor.selectionStart, editor.selectionEnd);
  findText(1);
}

/** Reemplaza todas las coincidencias como una sola edición deshacible. */
function replaceEveryText() {
  const result = EditorSearch.replaceAll(fullText(), $('search-text').value,
    $('replace-text').value, $('search-case').checked);
  if (result.count) insert(result.text, 0, editor.value.length);
  $('search-status').textContent = `${result.count} reemplazo${result.count === 1 ? '' : 's'}`;
}
/** Solicita una ruta, crea el archivo mediante la API y lo abre en el editor. */
async function create() {
  const path = prompt('Nuevo archivo dentro de src (ejemplo: utilidades/math.st):', 'programa.st');
  if (!path) return;
  const created = await api('create', {path}); await refresh(); await openFile(created.path);
}

/** Envía el archivo a la papelera tras confirmar su ruta y los cambios sin guardar. */
async function deleteFile() {
  if (!current || saving || running || readOnly) return;
  const path = current, baseline = original;
  const pending = dirty() ? '\nLos cambios sin guardar se perderán.' : '';
  if (!confirm(`¿Mover ${path} a la papelera del proyecto?${pending}`)) return;
  // Bloquea guardar/ejecutar durante la operación y evita respuestas de aperturas antiguas.
  saving = true; render();
  try {
    const result = await api('delete', {path, original: baseline});
    if (current === path) {
      opening++; current = ''; original = ''; folded = new FoldDocument(); editor.value = ''; editor.disabled = true;
      $('filename').textContent = '[sin archivo]'; $('breadcrumb').textContent = 'Selecciona un archivo para comenzar';
      $('welcome').hidden = false; $('language').textContent = 'ST'; hideCompletion();
    }
    await refresh(); message('Archivo en papelera: ' + result.trash);
  } finally { saving = false; render(); }
}

/** Localiza cadenas y comentarios ST sin reconocer delimitadores dentro de otros bloques.
 * En un comentario abierto, el texto hasta EOF permanece protegido durante la escritura.
 */
function stProtectedAt(text, position) {
  let i = 0;
  while (i < text.length && i < position) {
    const start = i;
    let closed = false;
    if (text.startsWith('//', i) || text[i] === '#') {
      const newline = text.indexOf('\n', i);
      i = newline < 0 ? text.length : newline;
      if (position > start && position <= i) return true;
      i++; continue;
    }
    const delimiter = text.startsWith('/*', i) ? '*/' : text.startsWith('"""', i) ? '"""' : null;
    if (delimiter) {
      const end = text.indexOf(delimiter, i + (delimiter === '*/' ? 2 : 3));
      closed = end >= 0;
      i = closed ? end + delimiter.length : text.length;
    } else if (text[i] === '"' || text[i] === "'") {
      const quote = text[i++];
      while (i < text.length) {
        if (text[i] === '\\') { i = Math.min(i + 2, text.length); continue; }
        if (text[i++] === quote) { closed = true; break; }
      }
    } else { i++; continue; }
    if (position > start && (position < i || (position === i && !closed))) return true;
  }
  return false;
}

/** Sugiere palabras, plantillas y nombres declarados en el archivo actual. */
function complete(force = false) {
  if (editor.selectionStart !== editor.selectionEnd) return hideCompletion();
  const format = current.split('.').pop().toLowerCase();
  if (['html', 'css', 'js'].includes(format)) {
    options = WebLanguages.complete(editor.value, editor.selectionStart, format, force);
    choice = 0; drawCompletion(); return;
  }
  if (!canRun()) return hideCompletion();
  const before = editor.value.slice(0, editor.selectionStart);
  // Ofrece el cierre completo al escribir un delimitador ST. Python mantiene sus cadenas triples.
  if (current.endsWith('.st')) {
    const opening = before.endsWith('/*') ? '/*' : before.endsWith('"""') ? '"""' : null;
    if (opening) {
      const indent = before.split('\n').at(-1).match(/^\s*/)[0];
      const closing = opening === '/*' ? '*/' : '"""';
      // Solo propone plantillas cuando el delimitador no está dentro de otro comentario o cadena.
      const head = before.slice(0, -opening.length);
      if (!stProtectedAt(head, head.length)) {
        const lead = opening + '\n' + indent + '    ';
        options = [{name: 'Comentario multilínea ' + opening, prefix: opening,
          text: lead + '\n' + indent + closing, kind: 'plantilla', caret: lead.length}];
        choice = 0; drawCompletion(); return;
      }
    }
    if (stProtectedAt(editor.value, editor.selectionStart)) return hideCompletion();
  }
  const prefix = editor.value.slice(0, editor.selectionStart).match(/[A-Za-z_][A-Za-z_0-9]*$/)?.[0] || '';
  // Tras un alias de módulo estándar, sugiere sus funciones públicas incluso con prefijo vacío.
  if (current.endsWith('.st')) {
    const member = before.match(/([A-Za-z_][A-Za-z_0-9]*)\.([A-Za-z_0-9]*)$/);
    if (member) {
      const imports = [...editor.value.matchAll(/\bimportar\s+["']([^"']+)["'](?:\s+como\s+([A-Za-z_][A-Za-z_0-9]*))?\s*;/g)];
      const imported = imports.find(m => (m[2] || m[1]) === member[1] && !stProtectedAt(editor.value, m.index));
      const functions = imported && standardModules[imported[1]];
      if (functions) {
        options = functions.filter(f => f.name.startsWith(member[2])).slice(0,12)
          .map(f => ({name:f.name, prefix:member[2], text:f.name, kind:f.signature}));
        choice = 0; drawCompletion(); return;
      }
    }
  }
  if (!force && prefix.length < 1) return hideCompletion();
  const isPython = current.endsWith('.py');
  const activeWords = isPython ? pythonWords : keywords;
  const templates = isPython ? pythonSnippets : snippets;
  const declarations = new RegExp('\\b(?:' + declarationWords.join('|') + ')\\s+([A-Za-z_][A-Za-z_0-9]*)', 'g');
  const names = isPython ? [...editor.value.matchAll(/(?:\b(?:def|class)\s+|\b)([A-Za-z_][A-Za-z_0-9]*)(?=\s*(?:[=(]|:))/g)].map(m => m[1]) : declarationWords.length ? [...editor.value.matchAll(declarations)].map(m => m[1]) : [];
  options = [...new Set([...activeWords,...names])].filter(x => x.startsWith(prefix) && (force || x !== prefix)).slice(0,12).map(name => ({name, prefix, text:templates[name] || name, kind:templates[name] ? 'plantilla' : activeWords.includes(name) ? 'palabra' : 'símbolo'}));
  choice = 0; drawCompletion();
}
/** Mide el cursor con una copia invisible de la línea, respetando fuente y tabulaciones.
 * El textarea no expone coordenadas del cursor. La copia usa texto, nunca HTML del usuario.
 */
function caretCoordinates() {
  const style = getComputedStyle(editor);
  const before = editor.value.slice(0, editor.selectionStart);
  const lines = before.split('\n');
  const mirror = document.createElement('span');
  mirror.style.cssText = 'position:fixed;left:-100000px;top:0;visibility:hidden;white-space:pre;';
  for (const property of ['font', 'letterSpacing', 'tabSize', 'textTransform']) mirror.style[property] = style[property];
  mirror.textContent = lines.at(-1);
  document.body.append(mirror);
  const width = mirror.getBoundingClientRect().width;
  mirror.remove();
  const rect = editor.getBoundingClientRect();
  const lineHeight = parseFloat(style.lineHeight);
  return {
    x: rect.left + parseFloat(style.paddingLeft) + width - editor.scrollLeft,
    y: rect.top + parseFloat(style.paddingTop) + (lines.length - 1) * lineHeight - editor.scrollTop,
    lineHeight, rect
  };
}

/** Ancla el menú al cursor; lo coloca encima si falta espacio y limita sus dimensiones. */
function positionCompletion() {
  const box = $('completions');
  if (box.hidden || !options.length) return;
  const caret = caretCoordinates();
  const bounds = $('editor-wrap').getBoundingClientRect();
  // No dejamos un menú flotando en otra línea cuando el cursor sale de la vista.
  if (caret.x < caret.rect.left || caret.x > caret.rect.left + editor.clientWidth || caret.y < bounds.top || caret.y + caret.lineHeight > bounds.bottom) {
    box.style.visibility = 'hidden'; return;
  }
  box.style.visibility = 'visible';
  const gap = 3, margin = 4;
  const below = Math.max(0, bounds.bottom - caret.y - caret.lineHeight - gap - margin);
  const above = Math.max(0, caret.y - bounds.top - gap - margin);
  const useAbove = below < Math.min(180, box.scrollHeight) && above > below;
  box.style.maxHeight = Math.min(230, useAbove ? above : below) + 'px';
  box.style.maxWidth = Math.max(0, bounds.width - margin * 2) + 'px';
  const left = Math.max(margin, Math.min(caret.x - bounds.left, bounds.width - box.offsetWidth - margin));
  const top = useAbove ? caret.y - bounds.top - gap - box.offsetHeight : caret.y - bounds.top + caret.lineHeight + gap;
  box.style.left = left + 'px';
  box.style.top = Math.max(margin, top) + 'px';
}

/** Construye las sugerencias y conecta cada fila con la inserción elegida. */
function drawCompletion() {
  const box = $('completions'); box.replaceChildren(); box.hidden = !options.length;
  options.forEach((item,i) => {
    const button = document.createElement('button'); button.role='option'; button.setAttribute('aria-selected', String(i===choice));
    button.classList.toggle('selected', i===choice); button.textContent=item.name;
    const type = document.createElement('small'); type.textContent=item.kind; button.append(type);
    button.onmousedown = e => {e.preventDefault(); choice=i; acceptCompletion();}; box.append(button);
  });
  positionCompletion();
  box.querySelector('.selected')?.scrollIntoView({block:'nearest'});
}
/** Vacía la selección y oculta el menú para no capturar teclas fuera de él. */
function hideCompletion() { options=[]; $('completions').hidden=true; }
/** Sincroniza la vista después de un cambio sin convertir el plegado en una edición. */
function showFolded(cursor = 0) {
  const top = editor.scrollTop, left = editor.scrollLeft;
  editor.value = folded.project().text;
  editor.setSelectionRange(cursor, cursor);
  painted = null; render(); editor.scrollTop = top; editor.scrollLeft = left; syncScroll();
}
/** Dibuja números originales y controles accesibles en el margen del código. */
function renderFoldLines() {
  const ranges = new Map(foldRanges(fullText(), folded.language).map(f => [f.header, f]));
  const starts = [0]; for (let i=0; i<fullText().length; i++) if (fullText()[i] === '\n') starts.push(i+1);
  const lineNumbers = new Map(starts.map((offset, index) => [offset, index+1]));
  $('lines').replaceChildren();
  let offset = 0;
  for (const line of editor.value.split('\n')) {
    const originalOffset = folded.offset(offset), row = document.createElement('div');
    const number = document.createElement('span'); number.textContent = lineNumbers.get(originalOffset) || starts.length;
    row.className = 'fold-line'; row.append(number);
    if (ranges.has(originalOffset)) {
      const button = document.createElement('button'), closed = folded.folds.some(f => f.header === originalOffset);
      button.textContent = closed ? '▸' : '▾'; button.setAttribute('aria-expanded', String(!closed));
      button.title = `${closed ? 'Desplegar' : 'Plegar'} bloque de la línea ${number.textContent}`;
      button.setAttribute('aria-label', button.title);
      button.onclick = () => {
        const position = folded.offset(editor.selectionStart);
        folded.toggle(originalOffset); hideCompletion(); showFolded(folded.visible(position));
      };
      row.append(button);
    }
    $('lines').append(row); offset += line.length + 1;
  }
}
/** Reemplaza texto mediante el modelo completo; Tab y plantillas también pasan por aquí. */
function insert(text, start=editor.selectionStart, end=editor.selectionEnd) {
  editor.focus(); const cursor = folded.edit(start, end, text); showFolded(cursor);
}
$('unfold').onclick = () => {
  const position = folded.offset(editor.selectionStart);
  folded.folds = []; hideCompletion(); showFolded(position);
};
/** Reconcilia la edición del navegador con el modelo, anclando diferencias en la selección. */
function receiveEdit() {
  const old = folded.project().text, next = editor.value;
  const anchor = inputAnchor; inputAnchor = Infinity;
  if (old === next) return;
  let start = 0;
  while (start < Math.min(anchor, old.length, next.length) && old[start] === next[start]) start++;
  let end = old.length, tail = next.length;
  while (end > start && tail > start && old[end-1] === next[tail-1]) { end--; tail--; }
  const cursor = folded.edit(start, end, next.slice(start, tail)); showFolded(cursor);
}
// Deshacer se refiere al código real: alternar flechas nunca añade entradas al historial.
editor.addEventListener('beforeinput', event => {
  if (readOnly) { event.preventDefault(); return; }
  inputAnchor = editor.selectionStart;
  if (event.inputType === 'historyUndo' || event.inputType === 'historyRedo') {
    event.preventDefault(); folded.history(event.inputType === 'historyRedo'); showFolded();
  }
});
// Copiar una selección incluye los bloques interiores, igual que el documento guardado.
editor.addEventListener('copy', event => {
  if (editor.selectionStart === editor.selectionEnd) return;
  event.preventDefault(); event.clipboardData.setData('text/plain', fullText().slice(folded.offset(editor.selectionStart), folded.offset(editor.selectionEnd)));
});
editor.addEventListener('cut', event => {
  if (readOnly) { event.preventDefault(); return; }
  if (editor.selectionStart === editor.selectionEnd) return;
  event.preventDefault();
  const start = editor.selectionStart, end = editor.selectionEnd;
  const finish = start === 0 && end === editor.value.length ? fullText().length : folded.offset(end, false);
  event.clipboardData.setData('text/plain', fullText().slice(folded.offset(start), finish));
  insert('', start, end);
});
/** Sustituye únicamente el prefijo escrito por la palabra o plantilla seleccionada. */
function acceptCompletion() {
  const item = options[choice]; if (!item) return;
  const start = editor.selectionStart-item.prefix.length;
  insert(item.text,start,editor.selectionEnd);
  if (item.caret !== undefined) { editor.setSelectionRange(start+item.caret,start+item.caret); render(); }
  hideCompletion();
}
// Eventos del editor: texto, desplazamiento, cursor y cierre diferido de sugerencias.
editor.addEventListener('input', event => {if (!event.isComposing) {receiveEdit(); render(); complete();}});
// No interrumpir la composición de caracteres con teclados internacionales.
editor.addEventListener('compositionend', () => {receiveEdit(); render();});
editor.addEventListener('scroll', syncScroll);
editor.addEventListener('click', () => {hideCompletion(); render();});
editor.addEventListener('keyup', render);
// Recalcula la posición si cambia el tamaño de ventana o del panel de salida.
window.addEventListener('resize', positionCompletion);
new ResizeObserver(positionCompletion).observe($('editor-wrap'));
editor.addEventListener('blur', () => setTimeout(hideCompletion,150));
/** Consulta las capacidades declaradas sin cambiar ni autorizar operaciones. */
$('permissions').onclick = async () => {
  try {
    const result = await api('permissions');
    $('permissions-list').textContent = Object.entries(result.permisos)
      .map(([name, allowed]) => `${allowed ? 'Permitido' : 'Denegado'} · ${name}`).join('\n') || 'Ningún permiso avanzado declarado.';
    $('permissions-dialog').showModal();
  } catch (error) { message(error.message); }
};
// El autocierre siempre está activo, incluso si una versión anterior lo desactivó.
const autocierre = true;
// El menú tiene prioridad sobre Tab/Enter; sin menú, estas teclas aplican sangría.
editor.addEventListener('keydown', e => {
  if (readOnly) return;
  if ((e.ctrlKey || e.metaKey) && ['z','y'].includes(e.key.toLowerCase())) {
    e.preventDefault(); folded.history(e.shiftKey || e.key.toLowerCase() === 'y'); hideCompletion(); showFolded(); return;
  }
  if ((e.ctrlKey || e.metaKey) && e.code === 'Space') {e.preventDefault(); complete(true); return;}
  if (options.length && ['ArrowDown','ArrowUp','Enter','Tab','Escape'].includes(e.key)) {
    e.preventDefault();
    if (e.key === 'Escape') hideCompletion();
    else if (e.key === 'Enter' || e.key === 'Tab') acceptCompletion();
    else {choice=(choice+(e.key==='ArrowDown'?1:-1)+options.length)%options.length; drawCompletion();}
    return;
  }
  // Una única edición del modelo conserva deshacer y el contenido plegado.
  if (autocierre && !e.ctrlKey && !e.metaKey && !e.altKey && !e.isComposing) {
    const pair = AutoPairs.edit(editor.value, editor.selectionStart, editor.selectionEnd, e.key, current.endsWith('.py'));
    if (pair) {
      e.preventDefault();
      if (pair.text !== undefined) {
        // Envolver una selección plegada debe incluir también sus líneas ocultas.
        if (pair.selectionEnd !== undefined && pair.start !== pair.end) {
          const selected = fullText().slice(folded.offset(pair.start), folded.offset(pair.end));
          pair.text = pair.text[0] + selected + pair.text.at(-1);
          pair.selectionEnd = pair.start + 1 + selected.length;
        }
        insert(pair.text, pair.start, pair.end);
      }
      editor.setSelectionRange(pair.cursor, pair.selectionEnd ?? pair.cursor);
      hideCompletion(); syncScroll(); return;
    }
  }
  // Mover el cursor invalida el prefijo sobre el que se calcularon las sugerencias.
  if (['ArrowLeft','ArrowRight','Home','End','PageUp','PageDown'].includes(e.key)) hideCompletion();
  if (e.key === 'Tab') {e.preventDefault(); insert('    '); hideCompletion();}
  if (e.key === 'Enter') {
    e.preventDefault(); const line=editor.value.slice(0,editor.selectionStart).split('\n').at(-1);
    const indent=line.match(/^\s*/)[0] + ((line.trimEnd().endsWith('{') || (current.endsWith('.py') && line.trimEnd().endsWith(':')))?'    ':''); insert('\n'+indent); hideCompletion();
  }
});
/** Guarda el archivo antes de iniciar su compilación y ejecución en el servidor. */
async function run() {
  if (running || !canRun()) return;
  const path=current;
  if (!await save()) return;
  await api('run',{path}); inputReady=false; running=true; $('output').textContent='> Preparando ejecución…'; render();
}
/** Envía respuestas al stdin del programa; no interpreta órdenes ni cambia de carpeta. */
async function sendTerminalInput(eof = false) {
  if (!inputReady || sendingInput) return;
  const line = $('terminal-line');
  const text = line.value;
  if (eof && text) { message('Envía primero la respuesta escrita o borra el texto antes de cerrar la entrada.'); return; }
  sendingInput = true; render();
  try {
    await api('input', {text, eof});
    if (eof) inputReady = false;
    else line.value = '';
  } catch (error) { message(error.message); }
  finally { sendingInput = false; render(); if (inputReady) line.focus(); }
}
$('terminal-input').onsubmit = event => { event.preventDefault(); sendTerminalInput(); };
$('terminal-eof').onclick = () => sendTerminalInput(true);
$('terminal-line').addEventListener('keydown', event => {
  if (event.ctrlKey && event.key.toLowerCase() === 'd') { event.preventDefault(); sendTerminalInput(true); }
});

// Los botones reutilizan las mismas operaciones que los atajos y muestran sus errores.
$('delete').onclick=()=>deleteFile().catch(e=>message(e.message));
$('save').onclick=()=>save().catch(e=>message(e.message));
$('check').onclick=()=>checkCurrent().catch(e=>message(e.message));
$('search-open').onclick=()=>openSearch(false);
$('search-next').onclick=()=>findText(1);
$('search-previous').onclick=()=>findText(-1);
$('replace-one').onclick=replaceText;
$('replace-all').onclick=replaceEveryText;
$('search-close').onclick=()=>{$('search-panel').hidden=true; editor.focus();};
$('search-text').oninput=()=>findText(1);
$('search-text').onkeydown=event=>{
  if(event.key==='Enter'){event.preventDefault();findText(event.shiftKey?-1:1);}
  if(event.key==='Escape'){$('search-panel').hidden=true;editor.focus();}
};
$('run').onclick=()=>run().catch(e=>message(e.message));
$('stop').onclick=()=>api('stop').catch(e=>message(e.message));
$('refresh').onclick=()=>refresh().catch(e=>message(e.message));
$('new').onclick=$('welcome-new').onclick=()=>create().catch(e=>message(e.message));
// Limpiar oculta la salida ya recibida sin borrar el registro conservado por el servidor.
let cleared='';
$('clear').onclick=()=>{cleared=lastOutput; $('output').textContent='';};
let lastOutput='';
/** Consulta el proceso cada medio segundo; respeta el desplazamiento manual de la salida. */
async function poll() {
  try {
    const status=await api('status'); running=status.running; inputReady=Boolean(status.input_ready);
    if (status.output !== lastOutput) {
      const out=$('output'); const bottom=out.scrollTop+out.clientHeight>=out.scrollHeight-30;
      lastOutput=status.output;
      out.textContent=status.output.startsWith(cleared)?status.output.slice(cleared.length):status.output;
      if(bottom) out.scrollTop=out.scrollHeight;
    }
    $('execution').textContent=running?'EN EJECUCIÓN':'LISTO'; render();
  } catch(e) { $('execution').textContent='SIN CONEXIÓN'; }
  setTimeout(poll,500);
}
// Atajos globales disponibles incluso cuando el foco está fuera del editor.
window.addEventListener('keydown',e=>{
  // Atajo del navegador del IDE: no se envía como respuesta al programa ST.
  if(e.ctrlKey && e.altKey && e.key.toLowerCase()==='h'){e.preventDefault();location.href='/inicio.html';return;}
  if((e.ctrlKey||e.metaKey)&&e.key.toLowerCase()==='s'){e.preventDefault();save().catch(e=>message(e.message));}
  if((e.ctrlKey||e.metaKey)&&e.shiftKey&&e.key.toLowerCase()==='b'){e.preventDefault();checkCurrent().catch(e=>message(e.message));}
  if((e.ctrlKey||e.metaKey)&&e.key.toLowerCase()==='f'){e.preventDefault();openSearch(false);}
  if((e.ctrlKey||e.metaKey)&&e.key.toLowerCase()==='h'){e.preventDefault();openSearch(true);}
  if(e.key==='F5'){e.preventDefault();run().catch(e=>message(e.message));}
});
// El navegador pide confirmación al salir si queda contenido sin guardar.
window.addEventListener('beforeunload',e=>{if(dirty()){e.preventDefault();e.returnValue='';}});
/** Oculta únicamente el explorador; conserva el archivo abierto y sus cambios. */
function toggleExplorer() {
  const hidden = document.body.classList.toggle('explorer-hidden');
  $('explorer').setAttribute('aria-hidden', String(hidden));
  positionCompletion();
}
window.addEventListener('keydown', event => {
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'b') {
    event.preventDefault(); toggleExplorer();
  }
});
/** Limita la terminal dejando espacio para editar; actualiza el separador accesible. */
function resizeTerminal(height) {
  const maximum = Math.max(110, document.querySelector('.workspace').clientHeight - 190);
  const value = Math.max(110, Math.min(maximum, height));
  $('terminal-panel').style.height = value + 'px';
  $('terminal-resizer').setAttribute('aria-valuemax', String(maximum));
  $('terminal-resizer').setAttribute('aria-valuenow', String(Math.round(value)));
  positionCompletion();
}
// Captura el puntero para continuar arrastrando aunque salga del borde o del panel.
let terminalDrag = null;
$('terminal-resizer').onpointerdown = event => {
  if (event.button !== 0) return;
  event.preventDefault();
  terminalDrag = {id:event.pointerId, y:event.clientY, height:$('terminal-panel').getBoundingClientRect().height};
  $('terminal-resizer').setPointerCapture(event.pointerId);
};
$('terminal-resizer').onpointermove = event => {
  if (terminalDrag && event.pointerId === terminalDrag.id) resizeTerminal(terminalDrag.height + terminalDrag.y - event.clientY);
};
$('terminal-resizer').onpointerup = $('terminal-resizer').onpointercancel = () => {terminalDrag = null;};
$('terminal-resizer').onlostpointercapture = () => {terminalDrag = null;};
$('terminal-resizer').onkeydown = event => {
  if (['ArrowUp','ArrowDown','Home','End'].includes(event.key)) {
    event.preventDefault();
    resizeTerminal(event.key === 'Home' ? 110 : event.key === 'End' ? Infinity :
      $('terminal-panel').getBoundingClientRect().height + (event.key === 'ArrowUp' ? 24 : -24));
  }
};
window.addEventListener('resize', () => resizeTerminal($('terminal-panel').getBoundingClientRect().height));
resizeTerminal($('terminal-panel').getBoundingClientRect().height);

// Inicio: carga el árbol y activa consultas independientes de ejecución y vocabulario.
refresh().then(async initialFile => { if (initialFile) await openFile(initialFile); render(); }).catch(e=>message(e.message)); poll();

refreshLanguage();

// La carga de Python no depende del éxito de la lectura del vocabulario ST.
loadPython();
