/** Prueba los controles reales del IDE sin modificar archivos ni ejecutar programas. */
const fs = require('fs'), vm = require('vm'), assert = require('assert');
const code = fs.readFileSync(__dirname + '/app.js', 'utf8');
const elements = new Map(), classes = new Set(), events = {};
function element(id) {
  if (!elements.has(id)) elements.set(id, {style:{}, attributes:{},
    setAttribute(key,value){this.attributes[key]=value;}, setPointerCapture(){},
    getBoundingClientRect(){return {height:parseFloat(this.style.height)||180};}});
  return elements.get(id);
}
const context = vm.createContext({$:element,positionCompletion(){},
  document:{body:{classList:{toggle(name){if(classes.has(name)){classes.delete(name);return false;}classes.add(name);return true;}}},querySelector:()=>({clientHeight:700})},
  window:{addEventListener(name,callback){(events[name] ||= []).push(callback);}}});
vm.runInContext(code.slice(code.indexOf('/** Oculta únicamente el explorador'),code.indexOf('// Inicio: carga el árbol')),context);
context.toggleExplorer(); assert.ok(classes.has('explorer-hidden'));
assert.equal(element('explorer').attributes['aria-hidden'],'true');
context.toggleExplorer(); assert.ok(!classes.has('explorer-hidden'));
context.resizeTerminal(9999); assert.equal(element('terminal-panel').style.height,'510px');
context.resizeTerminal(-1); assert.equal(element('terminal-panel').style.height,'110px');
context.resizeTerminal(200);
const resizer=element('terminal-resizer');
resizer.onpointerdown({button:0,pointerId:1,clientY:500,preventDefault(){}});
resizer.onpointermove({pointerId:1,clientY:400});
assert.equal(element('terminal-panel').style.height,'300px');
resizer.onpointerup(); resizer.onpointermove({pointerId:1,clientY:200});
assert.equal(element('terminal-panel').style.height,'300px');
resizer.onkeydown({key:'ArrowUp',preventDefault(){}}); assert.equal(element('terminal-panel').style.height,'324px');
console.log('IDE: explorador, arrastre superior, límites y control por teclado correctos.');
