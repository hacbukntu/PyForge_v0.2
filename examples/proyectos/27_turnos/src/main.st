// Rotación de turnos
// Asigna personal por rotación circular para evitar una asignación fija.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

variable personas = ["Ana","Luis","Eva"];
mostrar("Día inicial (0..30):");
variable inicio = entero(entrada(""));
si inicio < 0 o inicio > 30 { lanzar "Día inválido"; }
para dia en rango(inicio, inicio+5) {
    mostrar("Día " + dia + ": " + personas[dia % longitud(personas)]);
}
