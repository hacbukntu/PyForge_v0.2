/** Temas: no admite valores arbitrarios ni almacena contraseñas como preferencias. */
const fs=require('fs'), vm=require('vm'), assert=require('assert');
const source=fs.readFileSync(__dirname+'/preferences.js','utf8');
for (const stored of ['normal','hacker','pro','<script>']) {
  const theme={value:'',onchange:null}, dataset={}, values={};
  const context=vm.createContext({document:{documentElement:{dataset},getElementById:id=>id==='theme'?theme:null},localStorage:{getItem:()=>stored,setItem:(k,v)=>{values[k]=v;}}});
  vm.runInContext(source,context);
  assert.equal(theme.value,stored==='<script>'?'normal':stored);
  theme.value='hacker';theme.onchange();assert.equal(dataset.theme,'hacker');
  assert.deepEqual(values,{'st-theme':'hacker'});
}
console.log('Temas Normal/Hacker/Pro: selección y persistencia limitadas.');
