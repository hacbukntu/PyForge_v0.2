/* Colores de JSON, TOML y SQL: análisis visual, nunca ejecución del contenido. */
(function (root) {
  'use strict';
  /** Escapa todos los fragmentos, incluidos los que no tienen categoría. */
  const escape = text => text.replace(/[&<>"']/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
  /** Conserva el texto exacto y tolera documentos incompletos mientras se escriben. */
  function highlight(text, format) {
    const patterns = {
      json: /"(?:\\[\s\S]|[^"\\])*"?|\b(?:true|false|null)\b|-?\b\d+(?:\.\d+)?(?:[eE][+-]?\d+)?\b/g,
      toml: /"""[\s\S]*?(?:"""|$)|'''[\s\S]*?(?:'''|$)|"(?:\\.|[^"\\\n])*"?|'[^'\n]*'?|#[^\n]*|\b(?:true|false|inf|nan)\b|[+-]?\b\d[\w.:+-]*|[\p{L}_][\p{L}\p{N}_-]*/gu,
      sql: /--[^\n]*|\/\*[\s\S]*?(?:\*\/|$)|'(?:''|[^'])*'|"(?:""|[^"])*"|\b\d+(?:\.\d+)?\b|\b[A-Za-z_][A-Za-z_0-9]*\b/g
    };
    const pattern = patterns[format];
    if (!pattern) return escape(text);
    let result = '', last = 0;
    for (const match of text.matchAll(pattern)) {
      const word = match[0], after = text.slice(match.index + word.length);
      let category = '';
      if ((format === 'toml' && word.startsWith('#')) || (format === 'sql' && /^(--|\/\*)/.test(word))) category = 'comment';
      else if (/^["']/.test(word)) category = format === 'json' && /^\s*:/.test(after) || format === 'toml' && /^\s*(?:\.|=)/.test(after) || format === 'sql' && word.startsWith('"') ? 'variable' : 'string';
      else if (/^(true|false|null)$/i.test(word)) category = 'boolean';
      else if (/^[+-]?\d/.test(word) || /^(inf|nan)$/.test(word)) category = 'number';
      else if (format === 'sql' && /^(CREATE|TABLE|VIEW|INDEX|UNIQUE|PRIMARY|KEY|FOREIGN|REFERENCES|NOT|NULL|DEFAULT|CHECK|CONSTRAINT|SELECT|FROM|WHERE|INSERT|INTO|VALUES|INTEGER|REAL|TEXT|BLOB|AUTOINCREMENT)$/i.test(word)) category = 'keyword';
      else category = 'variable';
      result += escape(text.slice(last, match.index)) + `<span class="${category}">${escape(word)}</span>`;
      last = match.index + word.length;
    }
    return result + escape(text.slice(last));
  }
  root.FileFormats = {highlight};
  if (typeof module !== 'undefined') module.exports = root.FileFormats;
})(typeof globalThis !== 'undefined' ? globalThis : this);
