// Carrera de dados
// Simula una carrera con semilla elegida por el usuario y resultados reproducibles.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

// Generador congruencial de demostración, no apto para contraseñas ni apuestas.
funcion siguiente(semilla: entero) -> entero { retornar (semilla * 17 + 3) % 97; }
mostrar("Semilla entre 0 y 96:");
variable semilla = entero(entrada(""));
si semilla < 0 o semilla > 96 { lanzar "Semilla fuera de rango"; }
variable distancia = 0;
para turno en rango(0, 3) {
    semilla = siguiente(semilla);
    variable dado = semilla % 6 + 1;
    distancia += dado;
    mostrar("Dado: " + dado);
}
mostrar("Distancia: " + distancia);
