/* Presentación y teclado. El navegador nunca calcula ni envía puntuaciones o posiciones. */
(() => {
  const canvas = document.getElementById('snake-board'), ctx = canvas.getContext('2d');
  const status = document.getElementById('snake-status');
  let direction = 0, paused = true, pending = false, timer;
  /** Dibuja únicamente el estado confirmado por el ejecutable ST. */
  function draw(state) {
    ctx.fillStyle = '#111820'; ctx.fillRect(0, 0, 400, 400);
    for (const [cells,color] of [[state.cuerpo,'#f3d44e'],[[state.comida],'#e66f62']]) {
      ctx.fillStyle = color;
      for (const cell of cells) ctx.fillRect((cell % 20)*20+1, Math.floor(cell/20)*20+1,18,18);
    }
    status.textContent = `Puntuación: ${state.puntos} · ${state.fin ? 'Partida terminada' : 'Jugando'}`;
    if (state.fin) paused = true;
  }
  /** Una sola petición activa evita solapar movimientos o invertir su orden. */
  async function step(action) {
    if (pending) return;
    clearTimeout(timer); pending = true;
    try {
      const response = await fetch('/api/snake/', {method:'POST',headers:{'Content-Type':'application/json','X-CSRFToken':document.querySelector('[name=csrfmiddlewaretoken]').value},body:JSON.stringify({accion:action,direccion:direction})});
      const data = await response.json();
      if (!response.ok) throw new Error(data.error || 'No se pudo avanzar');
      draw(data);
    } catch (error) { paused = true; status.textContent = error.message; }
    finally { pending = false; if (!paused) timer = setTimeout(() => step('paso'),250); }
  }
  document.getElementById('snake-start').onclick = () => { if (pending) return; direction=0; paused=false; canvas.focus(); step('reiniciar'); };
  document.getElementById('snake-pause').onclick = () => { paused=!paused; clearTimeout(timer); if (!paused && !pending) step('paso'); };
  canvas.addEventListener('keydown', event => {
    const key = {ArrowRight:0,d:0,ArrowDown:1,s:1,ArrowLeft:2,a:2,ArrowUp:3,w:3}[event.key];
    if (key !== undefined) { event.preventDefault(); direction=key; }
  });
})();
