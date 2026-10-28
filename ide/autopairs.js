/* Autocierre independiente del DOM: produce una edición que aplica FoldDocument.
 * No cambia el texto oculto ni administra otro historial de deshacer. */
(function (root) {
  'use strict';
  /** Localiza comentarios y cadenas hasta el cursor, incluidos bloques anidados ST. */
  function context(text, end, python) {
    let quote = '', block = 0, line = false;
    for (let i = 0; i < end; i++) {
      const c = text[i], pair = text.slice(i, i + 2), triple = text.slice(i, i + 3);
      if (line) { if (c === '\n') line = false; continue; }
      if (block) {
        if (pair === '/*') { block++; i++; }
        else if (pair === '*/') { block--; i++; }
        continue;
      }
      if (quote) {
        if (c === '\\') { i++; continue; }
        if (text.slice(i, i + quote.length) === quote) { i += quote.length - 1; quote = ''; }
        continue;
      }
      if (c === '#' || (!python && pair === '//')) { line = true; continue; }
      if (!python && pair === '/*') { block = 1; i++; continue; }
      if (triple === '"""' || (python && triple === "'''")) { quote = triple; i += 2; }
      else if (c === '"' || c === "'") quote = c;
    }
    return {quote, protected: !!(quote || block || line)};
  }
  /** Devuelve null si corresponde la edición normal del navegador. */
  function edit(text, start, end, key, python = false) {
    const pairs = {'(': ')', '[': ']', '{': '}', '"': '"', "'": "'"};
    const state = context(text, start, python);
    // La tercera comilla crea el bloque triple que ya admite ST/Python.
    if (start === end && (key === '"' || (python && key === "'")) &&
        text.slice(Math.max(0, start - 2), start) === key.repeat(2) &&
        !context(text, start - 2, python).protected) {
      return {start, end, text: key.repeat(4), cursor: start + 1};
    }
    if (start === end && state.quote === key.repeat(3) &&
        text.slice(start, start + 3) === state.quote) return {cursor: start + 3};

    // Permite saltar el cierre de una cadena; no actúa sobre comentarios.
    if (start === end && text[start] === key && ')]}\"\''.includes(key) &&
        (!state.protected || state.quote === key)) return {cursor: start + 1};
    if (key === 'Backspace' && start === end && start > 0 && pairs[text[start - 1]] === text[start]) {
      if (!context(text, start - 1, python).protected) return {start: start - 1, end: start + 1, text: '', cursor: start - 1};
    }
    if (state.protected || !pairs[key]) return null;
    // No inserta comillas de cierre en medio de un identificador o un literal.
    if (start === end && /[\p{L}\p{N}_]/u.test(text[start] || '')) return null;
    return {start, end, text: key + text.slice(start, end) + pairs[key], cursor: start + 1, selectionEnd: end + 1};
  }
  root.AutoPairs = {edit, context};
  if (typeof module !== 'undefined') module.exports = root.AutoPairs;
})(typeof globalThis !== 'undefined' ? globalThis : this);
