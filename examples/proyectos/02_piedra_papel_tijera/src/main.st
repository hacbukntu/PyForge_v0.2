// Piedra, papel y tijera
// Juega tres rondas contra una secuencia de movimientos reproducible.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

// 0=piedra, 1=papel, 2=tijera. El bot rota sus movimientos.
mostrar("Tres jugadas: 0 piedra, 1 papel, 2 tijera.");
variable puntos = 0;
para ronda en rango(0, 3) {
    variable jugador = entero(entrada(""));
    si jugador < 0 o jugador > 2 { lanzar "Jugada fuera de rango"; }
    variable bot = ronda % 3;
    si jugador == bot { mostrar("Empate"); }
    sino_si (jugador - bot + 3) % 3 == 1 { puntos += 1; mostrar("Victoria"); }
    sino { mostrar("Derrota"); }
}
mostrar("Victorias: " + puntos);
