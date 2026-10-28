// Buscar coincidencias solapadas
// Encuentra todas las posiciones donde aparece un patrón corto.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

funcion coincide(texto: cadena, patron: cadena, inicio: entero) -> booleano {
    para j en rango(0, longitud(patron)) { si texto[inicio+j] != patron[j] { retornar falso; } }
    retornar verdadero;
}
variable texto = "bananana";
variable patron = "ana";
variable cantidad = 0;
para i en rango(0, longitud(texto)-longitud(patron)+1) {
    si coincide(texto,patron,i) { mostrar("Posición: " + i); cantidad += 1; }
}
mostrar("Total: " + cantidad);
