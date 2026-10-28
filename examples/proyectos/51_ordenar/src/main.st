// Ordenación por inserción
// Ordena una lista manteniendo los elementos de igual valor en su orden relativo.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

variable datos = [8,3,5,3,1];
para i en rango(1, longitud(datos)) {
    variable valor = datos[i];
    variable j = i;
    mientras j > 0 y datos[j-1] > valor { datos[j] = datos[j-1]; j -= 1; }
    datos[j] = valor;
}
para i en rango(0, longitud(datos)) { mostrar(datos[i]); }
