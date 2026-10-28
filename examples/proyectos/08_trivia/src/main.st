// Trivia con puntuación
// Corrige respuestas, calcula puntos y presenta el porcentaje de aciertos.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

variable preguntas = ["Capital de Francia", "Resultado de 6 por 7", "Extension de ST"];
variable respuestas = ["Paris", "42", ".st"];
variable puntos = 0;
para i en rango(0, longitud(preguntas)) {
    mostrar(preguntas[i]);
    si entrada("") == respuestas[i] { puntos += 1; mostrar("Correcto"); } sino { mostrar("Respuesta: " + respuestas[i]); }
}
mostrar("Aciertos: " + puntos + "/3");
mostrar("Porcentaje: " + (puntos * 100 / 3));
