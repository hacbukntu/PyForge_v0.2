/** Anfitrión simulado: vocabulario sin fuentes, importaciones y rutas tratadas como datos. */
const assert=require('assert'), Module=require('module');
let captured, trusted=true;
const text='importar "juego";\njuego.ava';
const doc={uri:{scheme:'file',fsPath:'/tmp/proyecto con espacios/$(no-ejecutar).st'},languageId:'st',getText:()=>text,offsetAt:()=>text.length,save:async()=>true};
const vscode={workspace:{get isTrusted(){return trusted;},getConfiguration:()=>({get:(key,def)=>def}),getWorkspaceFolder:()=>undefined},window:{activeTextEditor:{document:doc},showWarningMessage:()=>{},showErrorMessage:()=>{}},CompletionItem:class {constructor(label){this.label=label;}},ProcessExecution:class {constructor(command,args,options){Object.assign(this,{command,args,options});}},Task:class {constructor(def,scope,name,source,execution){this.execution=execution;}},TaskScope:{Global:1},TaskRevealKind:{Always:1},tasks:{executeTask:async task=>{captured=task;}}};
const original=Module._load;Module._load=function(name,...args){return name==='vscode'?vscode:original.call(this,name,...args);};
const ext=require('../extensions/vscode-st/src/extension.js');Module._load=original;
(async()=>{
 assert(ext.completion(doc,{}).some(item=>item.label==='avanzar_serpiente'));
 await ext.runFile();assert.deepEqual(captured.execution.args,['run',doc.uri.fsPath]);
 assert.equal(captured.execution.command,'pyforgec');
 captured=null;trusted=false;await ext.runFile();assert.equal(captured,null);
 assert(!ext.codeOnly('// variable secreto\n"funcion falso"').includes('secreto'));
 console.log('Extensión: vocabulario incluido, rutas literales y confianza comprobados.');
})().catch(error=>{console.error(error);process.exitCode=1;});
