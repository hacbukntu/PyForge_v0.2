/** Casos de edición reales: posición, selección, comentarios, escapes y plegado. */
const assert = require('node:assert/strict');
const {edit} = require('./autopairs.js');
assert.deepEqual(edit('', 0, 0, '('), {start:0,end:0,text:'()',cursor:1,selectionEnd:1});
assert.equal(edit('// comentario',13,13,'('), null);
assert.equal(edit('/* bloque\n anidado /* x */',26,26,'"'), null);
assert.equal(edit('"cadena',7,7,'('), null);
assert.equal(edit('# comentario',12,12,'[',true), null);
assert.equal(edit('"""texto\n',9,9,'(',true), null);
assert.deepEqual(edit('()',1,1,')'),{cursor:2});
assert.deepEqual(edit('""',1,1,'"'),{cursor:2});
assert.deepEqual(edit('()',1,1,'Backspace'),{start:0,end:2,text:'',cursor:0});
assert.equal(edit('"()"',2,2,'Backspace'),null);
assert.deepEqual(edit('abc',0,3,'['),{start:0,end:3,text:'[abc]',cursor:1,selectionEnd:4});
assert.equal(edit('abc',1,1,'"'),null);
assert.equal(edit('"\\"texto',8,8,'('),null);
console.log('Autocierre: 12 comprobaciones correctas');
assert.deepEqual(edit('""',2,2,'"'),{start:2,end:2,text:'""""',cursor:3});
assert.deepEqual(edit('"""texto"""',8,8,'"'),{cursor:11});
