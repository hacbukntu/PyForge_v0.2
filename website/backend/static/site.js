/* Mejora progresiva: las validaciones reales siempre están en Django y ST. */
'use strict';
for (const form of document.querySelectorAll('form')) {
  form.addEventListener('submit', () => {
    const button = form.querySelector('button');
    if (button) { button.disabled = true; button.textContent = 'Enviando…'; }
  });
}
