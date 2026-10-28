/* Plegado visual: el documento completo vive en FoldDocument, nunca en el textarea reducido. */
'use strict';

/** Detecta llaves y comentarios multilínea ST sin interpretar delimitadores dentro de cadenas. */
function foldRanges(text, language) {
  const lines = text.split('\n'), starts = [0];
  for (let i = 0; i < text.length; i++) if (text[i] === '\n') starts.push(i + 1);
  const ranges = [], stack = [];
  let line = 0, quote = '', comment = '', opening = 0;
  /** Conserva la cabecera y el cierre; solo oculta líneas interiores completas. */
  const add = (first, last) => {
    if (last > first + 1) ranges.push({header:starts[first], start:starts[first + 1], end:starts[last]});
  };
  if (language === 'py') {
    // Indentación conservadora: omite cadenas triples, comentarios y continuaciones entre paréntesis.
    let triple = '', brackets = 0;
    const logical = [];
    for (let row = 0; row < lines.length; row++) {
      let code = '', localQuote = '';
      const value = lines[row], continued = brackets > 0 || triple !== '';
      for (let col = 0; col < value.length; col++) {
        const c = value[col], three = value.slice(col, col + 3);
        if (triple) { if (three === triple) { triple = ''; col += 2; } else if (c === '\\') col++; continue; }
        if (localQuote) { if (c === '\\') col++; else if (c === localQuote) localQuote = ''; continue; }
        if (c === '#') break;
        if (three === '"""' || three === "'''") { triple = three; col += 2; continue; }
        if (c === '"' || c === "'") { localQuote = c; code += 's'; continue; }
        code += c;
        if ('([{'.includes(c)) brackets++;
        if (')]}'.includes(c)) brackets = Math.max(0, brackets - 1);
      }
      if (!continued && code.trim()) logical.push({row, indent:value.match(/^\s*/)[0].replace(/\t/g, '        ').length, block:code.trimEnd().endsWith(':') && brackets === 0});
    }
    for (let n = 0; n < logical.length; n++) {
      const head = logical[n];
      if (!head.block || !logical[n + 1] || logical[n + 1].indent <= head.indent) continue;
      let next = n + 1;
      while (next < logical.length && logical[next].indent > head.indent) next++;
      const end = next < logical.length ? starts[logical[next].row] : text.length;
      if (end > starts[head.row + 1]) ranges.push({header:starts[head.row], start:starts[head.row + 1], end});
    }
    return ranges;
  }
  if (language !== 'st') return [];
  for (let i = 0; i < text.length; i++) {
    const c = text[i];
    if (c === '\n') { line++; if (comment === 'line') comment = ''; }
    if (comment === 'line') continue;
    if (comment) {
      if (text.startsWith(comment, i)) { add(opening, line); i += comment.length - 1; comment = ''; }
      continue;
    }
    if (quote) { if (c === '\\') { if (text[i+1] === '\n') line++; i++; } else if (c === quote) quote = ''; continue; }
    if (text.startsWith('/*', i)) { comment = '*/'; opening = line; i++; continue; }
    if (text.startsWith('"""', i)) { comment = '"""'; opening = line; i += 2; continue; }
    if (text.startsWith('//', i) || c === '#') { comment = 'line'; continue; }
    if (c === '"' || c === "'") { quote = c; continue; }
    if (c === '{') stack.push(line);
    if (c === '}' && stack.length) add(stack.pop(), line);
  }
  // Una cabecera puede abrir varios bloques: la flecha representa el exterior.
  return ranges.sort((a,b) => a.header - b.header || b.end - a.end).filter((r,i,all) => !i || r.header !== all[i-1].header);
}

/** Modelo de texto completo, proyección visible y deshacer independiente del estado visual. */
class FoldDocument {
  constructor(text = '', language = 'st') { this.text = text; this.language = language; this.folds = []; this.undo = []; this.redo = []; }
  /** Captura solo datos del documento: plegar no cuenta como una edición. */
  snapshot() { return {text:this.text, folds:this.folds.map(f => ({...f}))}; }
  /** Proyecta ocultando interiores; conserva una tabla de huecos para traducir posiciones. */
  project() {
    if (this.cachedText === this.text && this.cachedFolds === this.folds) return this.cachedProjection;
    let text = '', from = 0; const holes = [];
    for (const fold of [...this.folds].sort((a,b) => a.start - b.start || b.end - a.end)) {
      if (fold.start < from) continue;
      text += this.text.slice(from, fold.start);
      holes.push({...fold, at:text.length}); from = fold.end;
    }
    text += this.text.slice(from);
    this.cachedText = this.text; this.cachedFolds = this.folds;
    return this.cachedProjection = {text, holes};
  }
  /** En un hueco, derecha apunta al cierre y izquierda al inicio del interior oculto. */
  offset(position, right = true) {
    let delta = 0;
    for (const hole of this.project().holes) if (hole.at < position || (right && hole.at === position)) delta += hole.end - hole.start;
    return position + delta;
  }
  /** Traduce una posición del documento a la vista; posiciones ocultas quedan en el límite visible del bloque. */
  visible(position) {
    let delta = 0;
    for (const hole of this.project().holes) {
      if (position < hole.start) break;
      if (position < hole.end) return hole.at;
      delta += hole.end - hole.start;
    }
    return position - delta;
  }
  /** Alterna el bloque cuya cabecera tiene este desplazamiento original. */
  toggle(header) {
    if (this.folds.some(f => f.header === header)) this.folds = this.folds.filter(f => f.header !== header);
    else { const range = foldRanges(this.text, this.language).find(f => f.header === header); if (range) this.folds = [...this.folds, range]; }
  }
  /** Aplica un cambio visible al texto real; el borrado de un salto limítrofe no borra el bloque. */
  edit(start, end, replacement) {
    const first = this.offset(start), last = start === end ? first : (start === 0 && end === this.project().text.length ? this.text.length : this.offset(end, false));
    this.undo.push(this.snapshot()); if (this.undo.length > 50) this.undo.shift(); this.redo = [];
    const finish = Math.max(first, last), delta = replacement.length - (finish - first);
    this.text = this.text.slice(0, first) + replacement + this.text.slice(finish);
    this.folds = this.folds.filter(f => finish <= f.header || first >= f.end).map(f => first <= f.header ? {header:f.header+delta,start:f.start+delta,end:f.end+delta} : f);
    return this.visible(first + replacement.length);
  }
  /** Deshace/repite texto completo, incluidos los interiores que estaban plegados. */
  history(redo = false) {
    const from = redo ? this.redo : this.undo, to = redo ? this.undo : this.redo;
    if (!from.length) return false;
    to.push(this.snapshot()); const state = from.pop(); this.text = state.text; this.folds = state.folds; return true;
  }
}
if (typeof module !== 'undefined') module.exports = {foldRanges, FoldDocument};
