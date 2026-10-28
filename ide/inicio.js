/* Menú de inicio: acciones explícitas del usuario sobre el servidor local autenticado. */
'use strict';
const element = id => document.getElementById(id);
/** Comparte el token de sesión sin incluirlo en URLs ni enlaces de donaciones. */
async function request(route, body = {}) {
  const response = await fetch('/api/' + route, {method:'POST', headers:{'Content-Type':'application/json', 'X-ST-Token':document.querySelector('meta[name="st-token"]').content}, body:JSON.stringify(body)});
  const result = await response.json();
  if (!response.ok) throw new Error(result.error || 'No se pudo completar la operación.');
  return result;
}
/** Abre el explorador del sistema; cancelar no cambia de carpeta ni descarta datos. */
async function selectAction(name) {
  if (name === 'manual' || name === 'apoyar') { window.open(name === 'manual' ? '/manual.md' : '/apoyar.html', '_blank', 'noopener,noreferrer'); return; }
  const buttons = [...element('menu-options').querySelectorAll('button')];
  buttons.forEach(button => { button.disabled = true; });
  element('menu-error').textContent = 'Selecciona una carpeta en la ventana del sistema…';
  try {
    const result = await request('project-select', {create:name === 'crear'});
    if (!result.cancelled) location.href = '/';
    else element('menu-error').textContent = '';
  } catch (error) { element('menu-error').textContent = error.message; }
  finally { buttons.forEach(button => { button.disabled = false; }); }
}
/** Cada recarga consulta la configuración actual, sin reconstruir el compilador. */
request('start-menu').then(data => {
  element('menu-title').textContent = data.titulo;
  element('menu-message').textContent = data.mensaje;
  element('menu-project').textContent = 'Carpeta actual: ' + data.project;
  for (const option of data.opciones) {
    const button = document.createElement('button');
    button.textContent = option.texto;
    button.onclick = () => selectAction(option.accion);
    element('menu-options').append(button);
  }
}).catch(error => { element('menu-error').textContent = error.message; });
