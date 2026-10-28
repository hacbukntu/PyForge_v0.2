/* Navegación del curso: los contenidos proceden de la generación local del manual. */
'use strict';
const $ = selector => document.querySelector(selector);
let course, current = 0;
let completed = new Set();
try { completed = new Set(JSON.parse(localStorage.getItem('st-course-completed') || '[]')); } catch (_) { /* El curso funciona también sin almacenamiento local. */ }

/** Crea nodos de texto sin interpretar nombres ni metadatos como HTML. */
function element(tag, text, className) {
  const node = document.createElement(tag);
  if (text !== undefined) node.textContent = text;
  if (className) node.className = className;
  return node;
}

/** Lista lecciones y filtra también su contenido; la selección no altera el manual. */
function listLessons() {
  const query = $('#search').value.toLocaleLowerCase('es');
  const list = $('#lessons'); list.replaceChildren();
  course.lessons.forEach((lesson, index) => {
    if (!(lesson.title + lesson.text).toLocaleLowerCase('es').includes(query)) return;
    const button = element('button', (completed.has(lesson.id) ? '✓ ' : '') + lesson.title);
    button.setAttribute('aria-current', String(index === current));
    button.onclick = () => openLesson(index);
    list.append(button);
  });
  if (!list.children.length) list.append(element('p', 'No hay lecciones con esa búsqueda.', 'small'));
  $('#progress').textContent = `${course.lessons.filter(l => completed.has(l.id)).length} de ${course.lessons.length} lecciones marcadas como leídas`;
}

/** Muestra HTML escapado por el generador y controles independientes del contenido. */
function openLesson(index, focus = true) {
  current = index;
  const lesson = course.lessons[index];
  const url = new URL(location.href);
  url.searchParams.set("tema", lesson.id);
  history.replaceState(null, "", url);
  const article = $('#lesson');
  article.innerHTML = lesson.html;
  article.prepend(element('h2', lesson.title));
  const controls = element('div', undefined, 'lesson-controls');
  const done = element('button', completed.has(lesson.id) ? 'Marcar como pendiente' : 'He leído esta lección ✓');
  done.onclick = () => {
    if (completed.has(lesson.id)) completed.delete(lesson.id); else completed.add(lesson.id);
    try { localStorage.setItem('st-course-completed', JSON.stringify([...completed])); } catch (_) { /* Preferencias locales opcionales. */ }
    openLesson(index, false);
  };
  controls.append(done);
  if (index + 1 < course.lessons.length) {
    const next = element('button', 'Siguiente lección →');
    next.onclick = () => openLesson(index + 1);
    controls.append(next);
  }
  article.append(controls);
  listLessons();
  if (focus) article.focus({preventScroll: false});
}

/** Muestra el código como texto, sin ejecutar ni interpretar HTML de los ejemplos. */
function showExample() {
  const example = course.examples[Number($('#example-select').value)];
  if (!example) return;
  // El generador escapa cada token; el texto copiado conserva el código original.
  if (example.html !== undefined) $('#example-code').innerHTML = example.html;
  else $('#example-code').textContent = example.code;
  const download = $('#download-example');
  if (download) {
    download.href = example.url;
    download.download = example.name.split('/').pop();
  }
  $('#copy-status').textContent = '';
}

/** Presenta únicamente artefactos publicados por el generador y sus sumas válidas. */
function showDownloads() {
  const list = $('#downloads');
  const items = course.downloads || [];
  list.replaceChildren();
  if (!items.length) {
    list.append(element('p', 'No hay paquetes publicados en este momento.', 'small'));
    return;
  }
  for (const item of items) {
    const card = element('article', undefined, 'download-card');
    const name = element('h3', item.name);
    const extension = item.name.split('.').pop().toLowerCase();
    const platform = extension === 'deb' ? 'Paquete Debian · Linux' : extension === 'vsix' ? 'Extensión de Visual Studio Code' : 'Paquete de PyForge';
    const description = element('p', platform, 'small');
    const link = element('a', 'Descargar archivo ↓', 'button');
    link.href = item.url;
    link.download = item.name;
    card.append(name, description, link);
    if (/^[a-f0-9]{64}$/i.test(item.sha256 || '')) {
      const details = document.createElement('details');
      details.append(element('summary', 'Ver suma SHA-256'));
      details.append(element('code', item.sha256, 'checksum'));
      card.append(details);
    }
    list.append(card);
  }
}

/** Carga el catálogo estático; los errores dejan accesible el manual descargable. */
async function start() {
  try {
    const response = await fetch('curso.json', {cache: 'no-cache'});
    if (!response.ok) throw new Error('No se pudo cargar el curso');
    course = await response.json();
    // Cada página inicializa exclusivamente sus propios controles.
    if ($('#lesson')) {
      const params = new URLSearchParams(location.search);
      $('#search').value = params.get('buscar') || '';
      $('#search').oninput = listLessons;
      const selected = course.lessons.findIndex(lesson => lesson.id === params.get('tema'));
      openLesson(selected >= 0 ? selected : 0, false);
    }
    if ($('#keywords')) {
      course.vocabulary.keywords.forEach(item => {
        const word = element('a', item.word);
        word.href = 'curso.html?buscar=' + encodeURIComponent(item.word);
        word.title = 'Consultar ' + item.word + ' en el curso';
        $('#keywords').append(word);
      });
    }
    if ($('#example-select')) {
      course.examples.forEach((example, index) => {
        const option = element('option', example.name); option.value = index;
        $('#example-select').append(option);
      });
      $('#example-select').onchange = showExample; showExample();
      $('#copy').onclick = async () => {
        try { await navigator.clipboard.writeText($('#example-code').textContent); $('#copy-status').textContent = 'Código copiado.'; }
        catch (_) { $('#copy-status').textContent = 'Selecciona el código y cópialo con tu teclado.'; }
      };
    }
    if ($('#downloads')) showDownloads();
  } catch (_) {
    const target = $('#lesson') || $('#keywords') || $('#example-code') || $('#downloads');
    if (target) target.textContent = 'No se pudo cargar el contenido. Recarga la página o consulta el manual completo.';
  }
}
// La portada es estática y no necesita descargar el catálogo del curso.
if ($('#lesson') || $('#keywords') || $('#example-select') || $('#downloads')) start();
