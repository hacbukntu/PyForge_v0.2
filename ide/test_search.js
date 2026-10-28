const assert = require('assert');
const Search = require('./search.js');
assert.deepEqual(Search.find('uno DOS uno', 'uno', 1), {start:8,end:11});
assert.deepEqual(Search.find('uno DOS uno', 'dos', 0), {start:4,end:7});
assert.deepEqual(Search.find('uno DOS uno', 'uno', 8, -1), {start:0,end:3});
assert.equal(Search.find('texto', '', 0), null);
assert.deepEqual(Search.replaceAll('Aa aA', 'aa', 'x'), {text:'x x',count:2});
assert.deepEqual(Search.replaceAll('Aa aA', 'aa', 'x', true), {text:'Aa aA',count:0});
console.log('IDE: búsqueda, navegación circular y reemplazo completo correctos.');
