// Serie de Fibonacci iterativa
// Calcula una serie con memoria constante y limita el índice al rango i64.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

mostrar("Cantidad de términos (1..90):");
variable cantidad = entero(entrada(""));
si cantidad < 1 o cantidad > 90 { lanzar "Cantidad fuera del rango de demostración"; }
variable a = 0;
variable b = 1;
para i en rango(0, cantidad) {
    mostrar(a);
    variable siguiente = a+b; a = b; b = siguiente;
}
