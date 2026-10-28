/** Pruebas del JavaScript real del editor, sin servidor ni un DOM simulado completo.
 * Ejecutar con node ide/test_comments.js; también admite Node integrado en Electron.
 */
const fs = require('node:fs');
const vm = require('node:vm');
const assert = require('node:assert/strict');
const source = fs.readFileSync(__dirname + '/app.js', 'utf8');

/** Extrae funciones públicas completas; sus llaves finales comienzan en la primera columna. */
function implementation(name) {
  const match = source.match(new RegExp('function ' + name + '\\([^]*?^}', 'm'));
  assert.ok(match, 'Función disponible: ' + name);
  return match[0];
}
const context = vm.createContext({});
vm.runInContext(`
let current = 'programa.st', options = [], choice = 0;
let keywords = ['variable'], snippets = {}, declarationWords = ['variable'];
const keywordInfo = new Map([['variable', {category:'keyword'}]]);
const editor = {value:'', selectionStart:0, selectionEnd:0,
  setSelectionRange(a,b) { this.selectionStart=a; this.selectionEnd=b; }};
function canRun() { return true; }
function hideCompletion() { options=[]; }
function drawCompletion() {}
function render() {}
function insert(text,start,end) { editor.value=editor.value.slice(0,start)+text+editor.value.slice(end); }
${source.match(/^const escapeHtml = .+$/m)[0]}
${['stProtectedAt','colorize','complete','acceptCompletion'].map(implementation).join('\n')}
`, context);

/** Evalúa cada escenario en el mismo contexto de funciones que se usa en el IDE. */
function evaluate(code) { return vm.runInContext(code, context); }
for (const opening of ['/*', '"""']) {
  evaluate(`editor.value=${JSON.stringify('    '+opening)}; editor.selectionStart=editor.selectionEnd=editor.value.length; complete();`);
  assert.equal(evaluate('options.length'), 1);
  evaluate('acceptCompletion();');
  assert.equal(evaluate('editor.value'), '    '+opening+'\n        \n    '+(opening==='/*'?'*/':'"""'));
  assert.equal(evaluate('editor.selectionStart'), 4+opening.length+1+8);
}
for (const text of ['/* variable oculta', '"""\nvariable oculta', '// /*', '# """', '"texto /*', '"escapada \\" /*']) {
  evaluate(`editor.value=${JSON.stringify(text)}; editor.selectionStart=editor.selectionEnd=editor.value.length; complete(true);`);
  assert.equal(evaluate('options.length'), 0, text);
}
for (const text of ['/* cerrado */', '"""cerrado"""', '"cadena"', "'cadena'"]) {
  assert.equal(evaluate(`stProtectedAt(${JSON.stringify(text)}, ${text.length})`), false, text);
}
const colored = evaluate(`colorize('/* variable\\nmostrar(1); */\\n"""otro\\ncomentario"""\\nvariable nombre = "/* texto */";')`);
assert.equal((colored.match(/class="comment"/g)||[]).length, 2);
assert.equal((colored.match(/class="string"/g)||[]).length, 1);
assert.equal((colored.match(/class="keyword"/g)||[]).length, 1);
evaluate('current="programa.py"; editor.value="/*"; editor.selectionStart=editor.selectionEnd=2; complete();');
assert.equal(evaluate('options.length'), 0);
console.log('Comentarios IDE: colores, cierres, posición del cursor y exclusión de cadenas/Python correctos.');
