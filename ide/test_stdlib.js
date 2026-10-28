/** Ejecuta los helpers reales del editor sin navegador; comprueba alias y prefijos tras punto. */
const fs = require('fs'), vm = require('vm'), assert = require('assert');
const source = fs.readFileSync(__dirname + '/app.js', 'utf8');
const helpers = source.slice(source.indexOf('function stProtectedAt('), source.indexOf('function caretCoordinates('));
const context = vm.createContext({
  current:'main.st', canRun:()=>true, editor:{value:'',selectionStart:0,selectionEnd:0},
  standardModules:{math:[{name:'raiz',signature:'raiz(valor: flotante) -> flotante'}]},
  keywords:[],snippets:{},declarationWords:[], options:[],choice:0,
  hideCompletion(){ context.options=[]; }, drawCompletion(){}
});
vm.runInContext(helpers, context);
/** Sitúa el cursor al final y recoge el autocompletado producido por complete(). */
function suggest(text) {
  context.editor.value=text;
  context.editor.selectionStart=context.editor.selectionEnd=text.length;
  context.complete(); return Array.from(context.options, item=>item.name);
}
assert.deepEqual(suggest('importar "math"; math.'), ['raiz']);
assert.deepEqual(suggest('importar "math" como m; m.ra'), ['raiz']);
assert.deepEqual(suggest('importar "math"; math.zz'), []);
assert.deepEqual(suggest('// importar "math";\nmath.ra'), []);
assert.deepEqual(suggest('importar "math"; // math.ra'), []);
assert.deepEqual(suggest('importar "math"; variable s = "math.ra'), []);
console.log('Módulos IDE: alias, prefijos y exclusión de comentarios/cadenas correctos.');
