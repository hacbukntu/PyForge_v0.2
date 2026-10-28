// Adivina el número
// Practica búsqueda por pistas con un máximo de cinco intentos.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

// El secreto fijo permite probar el juego; cambia este valor para otra partida.
variable secreto = 37;
variable acierto = falso;
mostrar("Adivina del 1 al 100: cinco intentos.");
para turno en rango(0, 5) {
    variable intento = entero(entrada(""));
    si intento == secreto { acierto = verdadero; mostrar("Ganaste"); romper; }
    si intento < secreto { mostrar("Mayor"); } sino { mostrar("Menor"); }
}
si no acierto { mostrar("Secreto: " + secreto); }
