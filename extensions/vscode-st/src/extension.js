'use strict';
// Solo APIs públicas de VS Code; la ejecución no utiliza bash ni interpolación de rutas.
const fs = require('fs');
const path = require('path');
const vscode = require('vscode');
const bundled = require('../vocabulary.json');

/** Lee metadatos locales o usa la copia incluida cuando el compilador está instalado. */
function vocabulary(document) {
  const config = vscode.workspace.getConfiguration('pyforge', document.uri);
  const root = config.get('sourceRoot') || vscode.workspace.getWorkspaceFolder(document.uri)?.uri.fsPath;
  if (!root) return bundled;
  try {
    const lexer = fs.readFileSync(path.join(root,'src/st/lexer.rs'),'utf8');
    const keywords = [...lexer.matchAll(/\("([A-Za-z_]\w*)",\s*Keyword::/g)].map(m=>m[1]);
    if (!keywords.length) return bundled;
    const registry = fs.readFileSync(path.join(root,'src/st/stdlib.rs'),'utf8');
    const modules = {};
    for (const match of registry.matchAll(/"([\w.]+)"\s*=>\s*include_str!\("\.\.\/stdlib\/([\w]+)\.st"\)/g)) {
      const text=fs.readFileSync(path.join(root,'src/stdlib',match[2]+'.st'),'utf8');
      modules[match[1]]=[...text.matchAll(/^funcion\s+(\w+)\s*\(/gm)].map(m=>m[1]);
    }
    return {keywords,modules};
  } catch (_) { return bundled; }
}

/** Excluye comentarios/cadenas de los símbolos locales; conserva offsets del documento. */
function codeOnly(text) {
  return text.replace(/\/\*[\s\S]*?(?:\*\/|$)|"""[\s\S]*?(?:"""|$)|\/\/[^\n]*|#[^\n]*|"(?:\\.|[^"\\])*"?|'(?:\\.|[^'\\])*'?/g,m=>' '.repeat(m.length));
}

/** Sugiere funciones del módulo importado, palabras y nombres del archivo actual. */
function completion(document, position) {
  const all=document.getText(), offset=document.offsetAt(position), before=all.slice(0,offset);
  const data=vocabulary(document), code=codeOnly(all);
  const member=before.match(/(\w+)\.(\w*)$/);
  let names=data.keywords, prefix=before.match(/\w*$/)?.[0] || '';
  if (member) {
    const imports=[...all.matchAll(/\bimportar\s+"([\w.]+)"(?:\s+como\s+(\w+))?/g)];
    const imported=imports.find(m=>(m[2]||m[1])===member[1] && code.slice(m.index,m.index+8)==='importar');
    names=imported ? data.modules[imported[1]] || [] : []; prefix=member[2];
  } else if (/importar\s+"[^"\n]*$/.test(before)) names=Object.keys(data.modules);
  else {
    if (prefix && code.slice(offset-prefix.length,offset).trim()==='') return [];
    names=[...names,...[...code.matchAll(/\b(?:variable|funcion|clase)\s+(\w+)/g)].map(m=>m[1])];
  }
  return [...new Set(names)].filter(n=>n.startsWith(prefix)).map(name=>new vscode.CompletionItem(name));
}

/** Ejecuta el archivo guardado, no otro archivo encontrado por accidente en el proyecto. */
async function runFile() {
  if (!vscode.workspace.isTrusted) { vscode.window.showWarningMessage('Confía en el proyecto antes de ejecutar ST.'); return; }
  const document=vscode.window.activeTextEditor?.document;
  if (!document || document.uri.scheme!=='file' || document.languageId!=='st') { vscode.window.showErrorMessage('Abre un archivo .st guardado.'); return; }
  if (!await document.save()) return;
  const folder=vscode.workspace.getWorkspaceFolder(document.uri);
  const compiler=vscode.workspace.getConfiguration('pyforge',document.uri).get('compilerPath','pyforgec');
  const execution=new vscode.ProcessExecution(compiler,['run',document.uri.fsPath],{cwd:folder?.uri.fsPath || path.dirname(document.uri.fsPath)});
  const task=new vscode.Task({type:'pyforge',file:document.uri.fsPath},folder || vscode.TaskScope.Global,'Ejecutar ST','PyForge',execution);
  task.presentationOptions={reveal:vscode.TaskRevealKind.Always,clear:true};
  await vscode.tasks.executeTask(task);
}

/** Registra proveedores y libera todos sus recursos al desactivar la extensión. */
function activate(context) {
  context.subscriptions.push(vscode.languages.registerCompletionItemProvider({scheme:'file',language:'st'},{provideCompletionItems:completion},'.','"'));
  context.subscriptions.push(vscode.commands.registerCommand('pyforge.runFile',runFile));
}
module.exports={activate,completion,runFile,codeOnly};
