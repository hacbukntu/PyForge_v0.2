/** Regresiones del documento real: plegado, edición, límites de bloques y deshacer. */
const assert = require('assert');
const {foldRanges, FoldDocument} = require('./folding.js');
const text = 'funcion f() {\n    si verdadero {\n        mostrar("}"); // {\n    }\n}\nmostrar(2);\n';
const model = new FoldDocument(text);
assert.equal(foldRanges(text, 'st').length, 2);
model.toggle(0);
assert.equal(model.text, text);
assert.equal(model.project().text, 'funcion f() {\n}\nmostrar(2);\n');
let visible = model.project().text;
const at = visible.indexOf('2');
model.edit(at, at+1, '3');
assert.equal(model.folds.length, 1);
assert.equal(model.text, text.replace('mostrar(2)', 'mostrar(3)'));
model.history(); assert.equal(model.text, text);
model.history(true); assert.ok(model.text.endsWith('mostrar(3);\n'));
model.toggle(0); assert.equal(model.project().text, model.text);
model.toggle(0);
const boundary = model.project().holes[0].at;
model.edit(boundary-1,boundary,'');
assert.ok(model.text.includes('mostrar("}")')); // Borrar junto al hueco no borra el interior.
assert.equal(model.folds.length, 0);
model.history(); assert.equal(model.folds.length,1);
model.edit(0,model.project().text.length,'nuevo');
assert.equal(model.text,'nuevo'); model.history(); assert.ok(model.text.includes('mostrar("}")'));
const nested = new FoldDocument(text);
nested.toggle(text.indexOf('    si')); nested.toggle(0); nested.toggle(0);
assert.equal(nested.folds.length,1);
assert.ok(!nested.project().text.includes('mostrar("}")'));
const comments='/*\n { falso }\n*/\nvariable s="{ ";\n"""\n comentario\n"""';
assert.equal(foldRanges(comments,'st').length,2);
assert.equal(foldRanges('funcion f() {\n sin_cierre','st').length,0);
const python='class A:\n    def f(self):\n        return "texto: #"\n\nprint(1)\n';
const py = new FoldDocument(python,'py');
assert.equal(foldRanges(python,'py').length,2);
py.toggle(0); assert.equal(py.project().text,'class A:\nprint(1)\n');
assert.equal(py.text,python);
const preceding = new FoldDocument(text); preceding.toggle(0); preceding.edit(0,0,'// antes\n');
assert.equal(preceding.folds[0].header,9);
assert.ok(preceding.project().text.startsWith('// antes\nfuncion'));
assert.deepEqual(foldRanges(text,'md'),[]);
console.log('Plegado: rangos ST/Python, comentarios, edición, anidación, selección total e historial correctos.');

/** Verifica las funciones reales guardar/ejecutar: la API nunca recibe la proyección. */
const fs = require('fs'), vm = require('vm');
const app = fs.readFileSync(__dirname + '/app.js', 'utf8');
const saved = [], hidden = new FoldDocument(text); hidden.toggle(0);
const context = vm.createContext({readOnly:false,current:'main.st',saving:false,running:false,inputReady:false,original:text,
  fullText:()=>hidden.text,render(){},canRun:()=>true,$:()=>({textContent:''}),
  api:async(route,body)=>{saved.push({route,body});}});
vm.runInContext(app.slice(app.indexOf('async function save()'),app.indexOf('/** Solicita una ruta')),context);
vm.runInContext(app.slice(app.indexOf('async function run()'),app.indexOf('/** Envía respuestas al stdin')),context);
(async()=>{
  await context.run();
  assert.equal(saved[0].route,'save'); assert.equal(saved[0].body.text,text);
  assert.equal(saved[1].route,'run');
  console.log('Plegado: Guardar y Ejecutar envían el código completo, incluido el interior oculto.');
})().catch(error=>{console.error(error);process.exitCode=1;});

/** Simula input del navegador, incluida la alternativa sin evento beforeinput. */
const edited = new FoldDocument(text); edited.toggle(0);
const editContext = vm.createContext({folded:edited, editor:{value:edited.project().text},inputAnchor:Infinity,
  showFolded(){ editContext.editor.value=edited.project().text; }});
vm.runInContext(app.slice(app.indexOf('function receiveEdit()'),app.indexOf('// Deshacer se refiere')),editContext);
editContext.editor.value=editContext.editor.value.replace('mostrar(2)', 'mostrar(23)');
editContext.receiveEdit();
assert.equal(edited.text,text.replace('mostrar(2)', 'mostrar(23)'));
assert.equal(edited.folds.length,1);
editContext.inputAnchor=0; editContext.editor.value='// Nuevo\n'+editContext.editor.value;
editContext.receiveEdit();
assert.equal(edited.text,'// Nuevo\n'+text.replace('mostrar(2)', 'mostrar(23)'));
assert.equal(edited.folds.length,1);
