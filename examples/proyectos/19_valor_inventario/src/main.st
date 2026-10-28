// Valor de inventario
// Calcula el coste total del almacén y detecta existencias bajas.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

variable nombres = ["cable","teclado","raton"];
variable unidades = [15,3,8];
variable costos = [200,1800,900];
variable valor = 0;
para i en rango(0, longitud(nombres)) {
    valor += unidades[i] * costos[i];
    si unidades[i] < 5 { mostrar("Reponer: " + nombres[i]); }
}
mostrar("Valor en centavos: " + valor);
