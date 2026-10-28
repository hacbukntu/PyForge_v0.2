/* Temas y creación/apertura de bóvedas; valores limitados y contraseñas efímeras. */
'use strict';
const themes = ['normal','hacker','pro'];
try { const stored = localStorage.getItem('st-theme'); if (themes.includes(stored)) document.documentElement.dataset.theme = stored; } catch (_) {}
const theme = document.getElementById('theme');
if (theme) {
  theme.value = document.documentElement.dataset.theme || 'normal';
  theme.onchange = () => { if (themes.includes(theme.value)) { document.documentElement.dataset.theme = theme.value; try { localStorage.setItem('st-theme',theme.value); } catch (_) {} } };
}
const privacyForm = document.getElementById('vault-form');
if (privacyForm) document.getElementById('vault-mode').onchange = () => { document.getElementById('vault-confirm-label').hidden = document.getElementById('vault-mode').value !== 'crear'; };
if (privacyForm) privacyForm.onsubmit = async event => {
  event.preventDefault();
  const password = document.getElementById('vault-password');
  const status = document.getElementById('vault-status');
  const confirmation = document.getElementById('vault-confirm');
  if (document.getElementById('vault-mode').value === 'crear' && password.value !== confirmation.value) { status.textContent = 'Las contraseñas no coinciden.'; return; }
  const button = event.submitter; button.disabled = true;
  try {
    const response = await fetch('/api/vault-open',{method:'POST',headers:{'Content-Type':'application/json','X-ST-Token':document.querySelector('meta[name="st-token"]').content},body:JSON.stringify({password:password.value,create:document.getElementById('vault-mode').value==='crear'})});
    password.value = ''; confirmation.value = '';
    const result = await response.json();
    if (!response.ok) throw new Error(result.error);
    if (!result.cancelled) location.href = '/';
    else status.textContent = 'Cancelado.';
  } catch (error) { status.textContent = error.message; }
  finally { button.disabled = false; }
};
const lockButton = document.getElementById('lock-project');
if (lockButton) lockButton.onclick = async () => {
  if (typeof save === 'function' && typeof dirty === 'function' && dirty()) {
    try { if (!await save()) return; } catch (error) { alert(error.message); return; }
  }
  const response = await fetch('/api/lock',{method:'POST',headers:{'Content-Type':'application/json','X-ST-Token':document.querySelector('meta[name="st-token"]').content},body:'{}'});
  const result = await response.json();
  if (response.ok) location.href = '/'; else alert(result.error);
};
