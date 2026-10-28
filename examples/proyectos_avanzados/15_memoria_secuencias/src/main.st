importar "time";
importar "io";
// Presenta una secuencia sin borrar el historial: es una demostración, no un concurso.
// La pausa usa time.dormir en vez de un bucle que consuma CPU.
variable secuencias = ["2", "24", "241", "2413"];
variable superadas = 0;
para ronda en rango(0, longitud(secuencias)) {
    mostrar("Memoriza: " + secuencias[ronda]);
    time.dormir(200);
    mostrar("Repite la secuencia sin espacios.");
    variable respuesta = io.leer_linea("Respuesta: ");
    si respuesta != secuencias[ronda] { mostrar("Fin: secuencia incorrecta"); romper; }
    superadas = superadas + 1;
}
mostrar("Rondas superadas: " + superadas + "/4");
