/* Lenguajes del navegador: coloreado y sugerencias locales, sin ejecutar el texto. */
(function (root) {
  'use strict';
  const escape = value => value.replace(/[&<>"']/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
  const tags = 'html head body title meta link script style main header footer nav section article aside div span p a img button form label input textarea select option table thead tbody tr th td ul ol li h1 h2 h3 canvas dialog template video'.split(' ');
  const attrs = 'class id href src alt title name value type placeholder required disabled aria-label role method action for data-'.split(' ');
  const properties = 'color background background-color display position width height min-width max-width margin padding border border-radius box-shadow font-size font-family font-weight line-height gap grid-template-columns align-items justify-content flex flex-direction overflow opacity transform transition cursor'.split(' ');
  const words = 'const let var function return if else for of in while break continue class extends new this async await try catch finally throw import from export default switch case typeof null undefined true false document window console fetch JSON Math addEventListener querySelector querySelectorAll getElementById textContent createElement appendChild preventDefault setTimeout clearTimeout requestAnimationFrame'.split(' ');
  /** Etiqueta fragmentos sin perder espacios ni permitir inyección de HTML. */
  function tokens(text, pattern, classify) {
    let output = '', end = 0;
    for (const match of text.matchAll(pattern)) {
      output += escape(text.slice(end, match.index));
      output += `<span class="${classify(match[0], text.slice(match.index + match[0].length))}">${escape(match[0])}</span>`;
      end = match.index + match[0].length;
    }
    return output + escape(text.slice(end));
  }
  /** Reconoce cadenas completas y comentarios antes de buscar identificadores. */
  function highlight(text, format) {
    if (format === 'html') {
      let output = '', end = 0;
      // Dentro de script/style cambia de gramática y conserva los delimitadores HTML.
      const pattern = /<!--[\s\S]*?(?:-->|$)|<(script|style)\b[^>]*>[\s\S]*?<\/\1\s*>|<[^>]*>?|\{\{[\s\S]*?\}\}|\{%[\s\S]*?%\}/gi;
      for (const match of text.matchAll(pattern)) {
        const part = match[0]; output += escape(text.slice(end, match.index));
        if (part.startsWith('<!--')) output += `<span class="comment">${escape(part)}</span>`;
        else if (match[1]) {
          const start = part.indexOf('>') + 1, close = part.toLowerCase().lastIndexOf('</');
          output += highlight(part.slice(0, start), 'html') + highlight(part.slice(start, close), match[1].toLowerCase() === 'script' ? 'js' : 'css') + highlight(part.slice(close), 'html');
        } else output += tokens(part, /"[^"]*"?|'[^']*'?|[A-Za-z_][\w:-]*/g, (word, after) => /^["']/.test(word) ? 'string' : /^\s*=/.test(after) ? 'variable' : 'keyword');
        end = match.index + part.length;
      }
      return output + escape(text.slice(end));
    }
    const pattern = /\/\*[\s\S]*?(?:\*\/|$)|\/\/[^\n]*|"(?:\\[\s\S]|[^"\\])*"?|'(?:\\[\s\S]|[^'\\])*'?|`(?:\\[\s\S]|[^`\\])*`?|#[a-fA-F0-9]{3,8}\b|\b\d+(?:\.\d+)?(?:px|rem|em|vh|vw|ms|s)?\b|--[\w-]+|[A-Za-z_$][\w$-]*/g;
    return tokens(text, pattern, (word, after) => {
      if (/^\/[/\*]/.test(word)) return 'comment';
      if (/^["'`]/.test(word)) return 'string';
      if (/^(true|false|null|undefined)$/.test(word)) return 'boolean';
      if (/^[\d#]/.test(word)) return 'number';
      if (format === 'css') return properties.includes(word) || /^\s*:/.test(after) ? 'keyword' : 'variable';
      return words.includes(word) ? 'keyword' : /^\s*\(/.test(after) ? 'function' : 'variable';
    });
  }
  /** Omite sugerencias dentro de comentarios/cadenas; el contenido se conserva intacto. */
  function protectedAt(text, position, format) {
    const pattern = format === 'html' ? /<!--[\s\S]*?(?:-->|$)|"[^"]*"?|'[^']*'?/g : /\/\*[\s\S]*?(?:\*\/|$)|\/\/[^\n]*|"(?:\\.|[^"\\])*"?|'(?:\\.|[^'\\])*'?|`(?:\\.|[^`\\])*`?/g;
    for (const match of text.matchAll(pattern)) {
      const start = match.index, end = start + match[0].length;
      const closed = match[0].startsWith('<!--') ? match[0].endsWith('-->') : match[0].startsWith('/*') ? match[0].endsWith('*/') : match[0].startsWith('//') ? end < text.length : match[0].length > 1 && match[0].at(-1) === match[0][0];
      if (position > start && (position < end || position === end && !closed)) return true;
    }
    return false;
  }
  /** Completa vocabulario y nombres locales; no presume tipos ni ejecuta JavaScript. */
  function complete(text, position, format, force = false) {
    const before = text.slice(0, position);
    if (format === 'html') {
      const embedded = /<(script|style)\b[^>]*>([^]*?)$/i.exec(before);
      if (embedded && !new RegExp('</' + embedded[1], 'i').test(embedded[2])) return complete(embedded[2], embedded[2].length, embedded[1].toLowerCase() === 'script' ? 'js' : 'css', force);
    }
    if (protectedAt(text, position, format)) return [];
    const prefix = before.match(/[A-Za-z_$-][\w$-]*$/)?.[0] || '';
    if (!force && !prefix) return [];
    const code = text.replace(/\/\*[\s\S]*?(?:\*\/|$)|\/\/[^\n]*|"(?:\\.|[^"\\])*"?|'(?:\\.|[^'\\])*'?|`(?:\\.|[^`\\])*`?/g, ' ');
    const locals = format === 'js' ? [...code.matchAll(/\b(?:let|const|var|function|class)\s+([A-Za-z_$][\w$]*)/g)].map(m => m[1]) : format === 'css' ? [...code.matchAll(/(--[\w-]+)\s*:/g)].map(m => m[1]) : [];
    const active = format === 'html' ? /<\/?[\w-]*$/.test(before) ? tags : attrs : format === 'css' ? [...properties,'flex','grid','block','none','auto','relative','absolute','var','calc'] : words;
    return [...new Set([...locals,...active])].filter(name => name.startsWith(prefix) && (force || name !== prefix)).slice(0,12).map(name => ({name, prefix, text:name, kind:locals.includes(name) ? 'símbolo local' : format.toUpperCase()}));
  }
  root.WebLanguages = {highlight, complete};
  if (typeof module !== 'undefined') module.exports = root.WebLanguages;
})(typeof globalThis !== 'undefined' ? globalThis : this);
