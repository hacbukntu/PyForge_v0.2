/** Comprueba colores y conservación literal del texto, incluso HTML hostil. */
const assert = require('assert');
const {highlight} = require('./formats.js');
for (const format of ['json','toml','sql']) {
  const text = format === 'json' ? '{"nombre":"<script>","activo":true,"edad":20}' : format === 'toml' ? '[package]\nnombre = "<script>" # comentario\nactivo = true\nvalor = 20' : 'CREATE TABLE "datos" (id INTEGER, nombre TEXT); -- comentario';
  const html = highlight(text,format);
  assert(!html.includes('<script>'));
  const decoded = html.replace(/<\/?span[^>]*>/g,'').replace(/&quot;/g,'"').replace(/&#39;/g,"'").replace(/&lt;/g,'<').replace(/&gt;/g,'>').replace(/&amp;/g,'&');
  assert.equal(decoded,text);
  assert(html.includes('class="variable"'));
}
assert(highlight('{"x": true}', 'json').includes('class="boolean"'));
assert(highlight('x = """# no es comentario\ntexto"""', 'toml').includes('class="string"'));
console.log('JSON, TOML y SQL: colores y texto conservados.');
