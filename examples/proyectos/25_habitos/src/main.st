// Registro semanal de hábitos
// Cuenta días cumplidos y la mejor racha consecutiva.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

variable cumplido = [verdadero,verdadero,falso,verdadero,verdadero,verdadero,falso];
variable total = 0;
variable racha = 0;
variable mejor = 0;
para i en rango(0, longitud(cumplido)) {
    si cumplido[i] { total += 1; racha += 1; } sino { racha = 0; }
    si racha > mejor { mejor = racha; }
}
mostrar("Días cumplidos: " + total);
mostrar("Mejor racha: " + mejor);
