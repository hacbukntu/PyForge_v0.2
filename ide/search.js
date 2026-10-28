/** Búsqueda pura utilizada por el IDE y sus pruebas Node; nunca interpreta código. */
const EditorSearch = (() => {
  function normalized(value, sensitive) { return sensitive ? value : value.toLocaleLowerCase('es'); }
  function find(text, query, start, direction = 1, sensitive = false) {
    if (!query) return null;
    const haystack = normalized(text, sensitive), needle = normalized(query, sensitive);
    let index;
    if (direction < 0) {
      index = haystack.lastIndexOf(needle, Math.max(0, start - 1));
      if (index < 0) index = haystack.lastIndexOf(needle);
    } else {
      index = haystack.indexOf(needle, Math.max(0, start));
      if (index < 0) index = haystack.indexOf(needle);
    }
    return index < 0 ? null : {start:index, end:index + query.length};
  }
  function replaceAll(text, query, replacement, sensitive = false) {
    if (!query) return {text, count:0};
    let cursor = 0, count = 0, output = '';
    while (cursor <= text.length) {
      const match = find(text, query, cursor, 1, sensitive);
      if (!match || match.start < cursor) break;
      output += text.slice(cursor, match.start) + replacement;
      cursor = match.end; count++;
    }
    return {text: count ? output + text.slice(cursor) : text, count};
  }
  return {find, replaceAll};
})();
if (typeof module !== 'undefined') module.exports = EditorSearch;
