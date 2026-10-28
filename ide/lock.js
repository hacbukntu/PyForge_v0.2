/* El desbloqueo usa una cookie HttpOnly; no guarda contraseñas en localStorage. */
'use strict';
document.getElementById('unlock').onsubmit = async event => {
  event.preventDefault();
  const input = document.getElementById('password');
  try {
    const response = await fetch('/api/unlock',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({password:input.value})});
    input.value = '';
    const result = await response.json();
    if (!response.ok) throw new Error(result.error);
    location.href = '/';
  } catch (error) { document.getElementById('error').textContent = error.message; }
};
