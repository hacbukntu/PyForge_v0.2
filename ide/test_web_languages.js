/** Regresiones: texto hostil, lenguajes mezclados y sugerencias dentro del editor. */
const assert = require('assert');
const {highlight,complete} = require('./web_languages.js');
const decode = html => html.replace(/<\/?span[^>]*>/g,'').replace(/&quot;/g,'"').replace(/&#39;/g,"'").replace(/&lt;/g,'<').replace(/&gt;/g,'>').replace(/&amp;/g,'&');
for (const [language,text] of [['html','<!-- <script> -->\n<button class="x">Hola</button><script>const cuenta = 3;</script>'],['css',':root { --tono: #ffcc00; color: var(--tono); }'],['js','// const oculto = 0;\nconst contador = "<img>"; function sumar() { return 2; }']]) {
  const rendered=highlight(text,language);
  assert.equal(decode(rendered),text);
  assert(!rendered.includes('<script>'));
  assert(!rendered.includes('<img>'));
  assert(rendered.includes('class="'));
}
assert(complete('<but',4,'html').some(x=>x.name==='button'));
assert(complete('backg',5,'css').some(x=>x.name==='background'));
const js='const contador = 0;\ncont';
assert(complete(js,js.length,'js').some(x=>x.name==='contador'));
assert.equal(complete('// cont',7,'js').length,0);
assert.equal(complete('"cont',5,'js').length,0);
assert.equal(complete('<!-- but',8,'html').length,0);
assert(complete('<script>doc',11,'html').some(x=>x.name==='document'));
console.log('HTML/CSS/JavaScript: colores, texto escapado y autocompletado correctos.');
