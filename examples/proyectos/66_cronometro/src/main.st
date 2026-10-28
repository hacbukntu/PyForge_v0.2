// Cronómetro monotónico
// Mide una pausa corta y encapsula el instante inicial en un objeto.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

importar "time";
clase Cronometro {
    variable inicio: entero;
    // El reloj monotónico evita cambios del reloj civil durante la medición.
    funcion transcurrido() -> entero { retornar time.monotono() - yo.inicio; }
}
variable reloj = nuevo Cronometro(time.monotono());
time.dormir(5);
mostrar("Transcurrieron al menos 4 ms: " + (reloj.transcurrido() >= 4));
